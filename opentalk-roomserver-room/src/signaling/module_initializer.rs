// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    marker::PhantomData,
};

use opentalk_roomserver_signaling::signaling_module::{
    SignalingModule, SignalingModuleFeatureDescription, SignalingModuleInitData,
};
use opentalk_roomserver_types::{breakout::BREAKOUT_MODULE_ID, core::CORE_MODULE_ID};
use opentalk_types_common::{
    features::{
        CALL_IN_FEATURE_ID, FeatureId, GUESTS_ALLOWED_FEATURE_ID, STORAGE_UPGRADABLE_FEATURE_ID,
    },
    modules::ModuleId,
};

use super::{ModuleDispatcher, ModuleHandle};
use crate::signaling::CORE_MODULES;

/// A set of initialized modules that can used through their [`ModuleHandle`]
pub type Modules = HashMap<ModuleId, Box<dyn ModuleHandle>>;

pub struct ModuleRegistry {
    modules: HashMap<ModuleId, Box<dyn ModuleInitializer>>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            modules: HashMap::new(),
        };
        registry
            .modules
            .insert(CORE_MODULE_ID, Box::new(CoreModuleDummy));
        registry
            .modules
            .insert(BREAKOUT_MODULE_ID, Box::new(BreakoutRoomModuleDummy));
        registry
    }

    async fn init_module(
        &self,
        module_id: &ModuleId,
        init_data: SignalingModuleInitData,
    ) -> Option<Box<dyn ModuleHandle>> {
        self.modules.get(module_id)?.init_module(init_data).await
    }

    pub fn add_module<M: SignalingModule + Sync + 'static>(&mut self) {
        self.modules
            .insert(M::NAMESPACE, Box::new(ModuleInitializerImpl::<M>::new()));
    }

    /// Attempt to initialize all given modules
    ///
    /// A given module id might be unknown to the [`ModuleRegistry`] and can't be initialized. A
    /// module might also not be able to initialize with the given [`SignalingModuleInitData`].
    /// Uninitialized modules are returned in a separate list.
    pub(crate) async fn initialize_modules(
        &self,
        init_data: SignalingModuleInitData,
    ) -> (Modules, Vec<ModuleId>) {
        let mut initializers = Vec::new();

        for module_id in init_data.room_parameters.module_settings.ids() {
            if CORE_MODULES.contains(module_id) {
                // These modules are part of the room task and don't need to be initialized
                continue;
            }

            let init_data = init_data.clone();

            initializers
                .push(async move { (module_id, self.init_module(module_id, init_data).await) });
        }

        let mut modules = HashMap::new();
        let mut uninitialized = Vec::new();

        for (module_id, module) in futures::future::join_all(initializers).await {
            let module_id = module_id.clone();
            if let Some(module) = module {
                modules.insert(module_id, module);
            } else {
                uninitialized.push(module_id);
            }
        }

        (modules, uninitialized)
    }

    pub fn print<P: DescriptionPrinter>(&self, printer: &mut P) {
        let mut initializers: Vec<_> = self.modules.values().collect();
        initializers.sort_by_key(|initializer| initializer.module_id());

        for initializer in initializers {
            printer.print(initializer.as_ref());
        }
    }

    pub fn module_features(&self) -> BTreeMap<ModuleId, BTreeSet<FeatureId>> {
        self.modules
            .values()
            .map(|initializer| (initializer.module_id(), initializer.features()))
            .collect()
    }
}

impl Default for ModuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub trait DescriptionPrinter {
    fn print(&mut self, module_initializer: &dyn ModuleInitializer);
}

#[async_trait::async_trait]
pub trait ModuleInitializer: Sync + Send {
    async fn init_module(
        &self,
        init_data: SignalingModuleInitData,
    ) -> Option<Box<dyn ModuleHandle>>;

    fn description(&self) -> &'static str;

    fn feature_descriptions(&self) -> &[SignalingModuleFeatureDescription];

    fn module_id(&self) -> ModuleId;

    fn features(&self) -> BTreeSet<FeatureId>;

    fn example_config(&self) -> Option<String>;
}

struct ModuleInitializerImpl<M: SignalingModule + Sync> {
    phantom_data: PhantomData<M>,
}

impl<M: SignalingModule + Sync> ModuleInitializerImpl<M> {
    pub fn new() -> Self {
        Self {
            phantom_data: PhantomData,
        }
    }
}

#[async_trait::async_trait]
impl<M: SignalingModule + Sync + 'static> ModuleInitializer for ModuleInitializerImpl<M> {
    async fn init_module(
        &self,
        init_data: SignalingModuleInitData,
    ) -> Option<Box<dyn ModuleHandle>> {
        if let Some(module) = M::init(init_data) {
            Some(Box::new(ModuleDispatcher { module }))
        } else {
            tracing::debug!("`{}` module initializer returned none", M::NAMESPACE);
            None
        }
    }

    fn description(&self) -> &'static str {
        M::DESCRIPTION
    }

    fn feature_descriptions(&self) -> &[SignalingModuleFeatureDescription] {
        M::FEATURES
    }

    fn module_id(&self) -> ModuleId {
        M::MODULE_ID
    }

    fn features(&self) -> BTreeSet<FeatureId> {
        M::FEATURES.iter().map(|f| f.feature_id.clone()).collect()
    }

    fn example_config(&self) -> Option<String> {
        let config = M::example_config();
        let toml = toml::to_string_pretty(&config)
            .inspect_err(|err| {
                tracing::error!(
                    "Failed to serialize module {} config to TOML: {err}",
                    M::MODULE_ID
                )
            })
            .ok()?;

        Some(toml)
    }
}

struct CoreModuleDummy;

const CORE_FEATURES: &[SignalingModuleFeatureDescription; 3] = &[
    SignalingModuleFeatureDescription {
        feature_id: CALL_IN_FEATURE_ID,
        description: "Enables telephone call-in functionality for meetings.",
    },
    SignalingModuleFeatureDescription {
        feature_id: GUESTS_ALLOWED_FEATURE_ID,
        description: "Allow guest access. With this feature enabled, guests are allowed to join meetings through invite links. When turned off, no invite links are generated, and invite links that existed before will be invalid.",
    },
    SignalingModuleFeatureDescription {
        feature_id: STORAGE_UPGRADABLE_FEATURE_ID,
        description: "Communicates to the frontend that the user's storage can be upgraded. Frontend will then show the corresponding link to the account management if the user's storage is close to the limit. This feature is usually configured differently the tariffs. If a user has a tariff which already provides the maximum available storage space, then that feature should be disabled. For all other tariffs it should be on.",
    },
];

#[async_trait::async_trait]
impl ModuleInitializer for CoreModuleDummy {
    async fn init_module(
        &self,
        _init_data: SignalingModuleInitData,
    ) -> Option<Box<dyn ModuleHandle>> {
        None
    }

    fn description(&self) -> &'static str {
        "Handles core room functionality such as joining and leaving"
    }

    fn feature_descriptions(&self) -> &[SignalingModuleFeatureDescription] {
        CORE_FEATURES
    }

    fn module_id(&self) -> ModuleId {
        CORE_MODULE_ID
    }

    fn features(&self) -> BTreeSet<FeatureId> {
        BTreeSet::from_iter(
            self.feature_descriptions()
                .iter()
                .map(|description| &description.feature_id)
                .cloned(),
        )
    }

    fn example_config(&self) -> Option<String> {
        None
    }
}

struct BreakoutRoomModuleDummy;

#[async_trait::async_trait]
impl ModuleInitializer for BreakoutRoomModuleDummy {
    async fn init_module(
        &self,
        _init_data: SignalingModuleInitData,
    ) -> Option<Box<dyn ModuleHandle>> {
        None
    }

    fn description(&self) -> &'static str {
        "Handles breakout room functionality"
    }

    fn feature_descriptions(&self) -> &[SignalingModuleFeatureDescription] {
        &[]
    }

    fn module_id(&self) -> ModuleId {
        BREAKOUT_MODULE_ID
    }

    fn features(&self) -> BTreeSet<FeatureId> {
        BTreeSet::from_iter(
            self.feature_descriptions()
                .iter()
                .map(|description| &description.feature_id)
                .cloned(),
        )
    }

    fn example_config(&self) -> Option<String> {
        None
    }
}
