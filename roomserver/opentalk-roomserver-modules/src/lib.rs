// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: OpenTalk Team <mail@opentalk.eu>

use itertools::Itertools as _;
use opentalk_roomserver_module_automod::AutomodModule;
use opentalk_roomserver_module_chat::ChatModule;
use opentalk_roomserver_module_e2ee::E2eeModule;
use opentalk_roomserver_module_echo::EchoModule;
use opentalk_roomserver_module_excalidraw::ExcalidrawModule;
use opentalk_roomserver_module_legal_vote::LegalVoteModule;
use opentalk_roomserver_module_livekit::LiveKitModule;
use opentalk_roomserver_module_meeting_notes::MeetingNotesModule;
use opentalk_roomserver_module_meeting_report::MeetingReportModule;
use opentalk_roomserver_module_moderation::ModerationModule;
use opentalk_roomserver_module_polls::PollsModule;
use opentalk_roomserver_module_raise_hands::RaiseHandsModule;
use opentalk_roomserver_module_reaction::ReactionModule;
use opentalk_roomserver_module_recording::RecordingModule;
use opentalk_roomserver_module_shared_folder::SharedFolderModule;
use opentalk_roomserver_module_subroom_audio::SubroomAudioModule;
use opentalk_roomserver_module_timer::TimerModule;
use opentalk_roomserver_module_training_participation_report::TrainingParticipationReportModule;
use opentalk_roomserver_module_transcription::TranscriptionModule;
use opentalk_roomserver_room::{
    ModuleRegistry,
    signaling::{DescriptionPrinter, ModuleInitializer},
};
use opentalk_roomserver_signaling::signaling_module::SignalingModuleFeatureDescription;
pub use opentalk_roomserver_types::{
    breakout::BREAKOUT_MODULE_ID, core::CORE_MODULE_ID, error::ERROR_MODULE_ID,
};
pub use opentalk_roomserver_types_automod::AUTOMOD_MODULE_ID;
pub use opentalk_roomserver_types_chat::CHAT_MODULE_ID;
pub use opentalk_roomserver_types_e2ee::E2EE_MODULE_ID;
pub use opentalk_roomserver_types_echo::ECHO_MODULE_ID;
pub use opentalk_roomserver_types_legal_vote::LEGAL_VOTE_MODULE_ID;
pub use opentalk_roomserver_types_livekit::LIVEKIT_MODULE_ID;
pub use opentalk_roomserver_types_meeting_notes::MEETING_NOTES_MODULE_ID;
pub use opentalk_roomserver_types_meeting_report::MEETING_REPORT_MODULE_ID;
pub use opentalk_roomserver_types_moderation::MODERATION_MODULE_ID;
pub use opentalk_roomserver_types_polls::POLLS_MODULE_ID;
pub use opentalk_roomserver_types_raise_hands::RAISE_HANDS_MODULE_ID;
pub use opentalk_roomserver_types_recording::RECORDING_MODULE_ID;
pub use opentalk_roomserver_types_shared_folder::SHARED_FOLDER_MODULE_ID;
pub use opentalk_roomserver_types_subroom_audio::SUBROOM_AUDIO_MODULE_ID;
pub use opentalk_roomserver_types_timer::TIMER_MODULE_ID;
pub use opentalk_roomserver_types_training_participation_report::TRAINING_PARTICIPATION_REPORT_MODULE_ID;
pub use opentalk_roomserver_types_transcription::TRANSCRIPTION_MODULE_ID;
use opentalk_types_common::modules::ModuleId;

/// Initialize the registry with all modules that are available for meetings
pub fn setup_registry() -> ModuleRegistry {
    let mut module_registry = ModuleRegistry::new();
    module_registry.add_module::<AutomodModule>();
    module_registry.add_module::<ChatModule>();
    module_registry.add_module::<E2eeModule>();
    module_registry.add_module::<LegalVoteModule>();
    module_registry.add_module::<LiveKitModule>();
    module_registry.add_module::<MeetingNotesModule>();
    module_registry.add_module::<MeetingReportModule>();
    module_registry.add_module::<ModerationModule>();
    module_registry.add_module::<RecordingModule>();
    module_registry.add_module::<EchoModule>();
    module_registry.add_module::<PollsModule>();
    module_registry.add_module::<ReactionModule>();
    module_registry.add_module::<SharedFolderModule>();
    module_registry.add_module::<SubroomAudioModule>();
    module_registry.add_module::<TimerModule>();
    module_registry.add_module::<TrainingParticipationReportModule>();
    module_registry.add_module::<TranscriptionModule>();
    module_registry.add_module::<RaiseHandsModule>();
    module_registry.add_module::<ExcalidrawModule>();
    module_registry
}

/// Print the modules and their features.
///
/// ## Example Usage
///
/// ```rust
/// # use opentalk_roomserver_modules::{ListPrinter, setup_registry};
///
/// let registry = setup_registry();
/// registry.print(&mut ListPrinter);
/// ```
pub struct ListPrinter;

impl DescriptionPrinter for ListPrinter {
    fn print(&mut self, module_initializer: &dyn ModuleInitializer) {
        println!(
            "{}: [{}]",
            module_initializer.module_id(),
            module_initializer
                .feature_descriptions()
                .iter()
                .map(|f| format!("\"{}\"", f.feature_id))
                .join(", ")
        );
    }
}

/// Print a description of the modules and their features.
///
/// ## Example Usage
///
/// ```rust
/// # use opentalk_roomserver_modules::{MarkdownPrinter, setup_registry};
///
/// let registry = setup_registry();
/// registry.print(&mut MarkdownPrinter);
/// ```
pub struct MarkdownPrinter;

impl DescriptionPrinter for MarkdownPrinter {
    fn print(&mut self, module_initializer: &dyn ModuleInitializer) {
        println!(
            "## Module `{}`\n\n{}\n\n### Features\n\n{}",
            module_initializer.module_id(),
            module_initializer.description(),
            generate_features_documentation(
                &module_initializer.module_id(),
                module_initializer.feature_descriptions()
            )
        );

        if let Some(config) = module_initializer.example_config() {
            println!("### Example\n\n```toml\n{config}```\n");
        }
    }
}

fn generate_features_documentation(
    module_id: &ModuleId,
    features: &[SignalingModuleFeatureDescription],
) -> String {
    if features.is_empty() {
        return "This module does not provide any configurable features.\n".to_string();
    }

    format!(
        "The following features can be configured for the module. All features are enabled by default and can be disabled either [by configuration](https://docs.opentalk.eu/admin/controller/advanced/defaults/) or [by tariff](https://docs.opentalk.eu/admin/controller/advanced/tariffs/).\n\n{}\n",
        features
            .iter()
            .map(|feature| generate_feature_documentation(module_id, feature))
            .join("\n\n")
    )
}

fn generate_feature_documentation(
    module_id: &ModuleId,
    feature: &SignalingModuleFeatureDescription,
) -> String {
    format!(
        "#### `{}::{}`\n\n{}",
        module_id, feature.feature_id, feature.description
    )
}
