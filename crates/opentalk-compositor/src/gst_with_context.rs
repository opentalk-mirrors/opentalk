// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

#![allow(clippy::missing_errors_doc)]
use std::panic::Location;

use anyhow::{Context, Result};
use glib::object::{IsA, ObjectExt as _};
use gstreamer::{
    element_factory::ElementBuilder,
    prelude::{ElementExt, ElementExtManual, GstBinExt, GstBinExtManual, PadExt},
    Bin, Element, ElementFactory, GhostPad, Object, Pad, PadLinkSuccess, State, StateChangeSuccess,
};

pub trait GstBinErrorExt: IsA<Bin> {
    #[track_caller]
    fn add_many_with_context<E: IsA<Element>>(&self, elements: &[&E]) -> Result<()> {
        self.add_many(elements).with_context(|| {
            format!(
                "Unable to add all elements '{:?}' to bin '{}' in {}",
                elements.iter().map(|e| e.type_()).collect::<Vec<_>>(),
                self.type_(),
                Location::caller()
            )
        })
    }

    #[track_caller]
    fn add_with_context(&self, dest: &impl IsA<Element>) -> Result<()> {
        self.add(dest).with_context(|| {
            format!(
                "Unable to add element '{}' to bin '{}' in {}",
                dest.type_(),
                self.type_(),
                Location::caller()
            )
        })
    }

    #[track_caller]
    fn by_name_with_context(&self, name: &str) -> Result<Element> {
        self.by_name(name).with_context(|| {
            format!(
                "Unable to get element by name '{name}' inside '{}' in {}",
                self.type_(),
                Location::caller()
            )
        })
    }
}

impl<B: IsA<Bin>> GstBinErrorExt for B {}

pub trait GstElementBuilderErrorExt {
    #[track_caller]
    fn build_with_context(self) -> Result<Element>;
}

impl GstElementBuilderErrorExt for ElementBuilder<'_> {
    #[track_caller]
    fn build_with_context(self) -> Result<Element> {
        self.build()
            .with_context(|| format!("Unable to build element in {}", Location::caller()))
    }
}

pub trait GstElementErrorExt: IsA<Element> {
    #[track_caller]
    fn add_pad_with_context(&self, pad: &impl IsA<Pad>) -> Result<()> {
        self.add_pad(pad).with_context(|| {
            format!(
                "Unable to add pad '{}' to element '{}' in {}",
                pad.type_(),
                self.type_(),
                Location::caller()
            )
        })
    }

    #[track_caller]
    fn link_with_context(&self, dest: &impl IsA<Element>) -> Result<()> {
        self.link(dest).with_context(|| {
            format!(
                "Unable to link element '{self:?}' with '{dest:?}' in {}",
                Location::caller()
            )
        })
    }

    #[track_caller]
    fn link_many_with_context<E: IsA<Element>>(elements: &[&E]) -> Result<()> {
        Element::link_many(elements).with_context(|| {
            format!(
                "Unable to link all elements '{:?}' in {}",
                elements.iter().map(|e| e.type_()).collect::<Vec<_>>(),
                Location::caller()
            )
        })
    }

    #[track_caller]
    fn remove_pad_with_context(&self, pad: &impl IsA<Pad>) -> Result<()> {
        self.remove_pad(pad).with_context(|| {
            format!(
                "Unable to remove pad '{}' from '{}' in {}",
                pad.type_(),
                self.type_(),
                Location::caller()
            )
        })
    }

    #[track_caller]
    fn request_pad_simple_with_context(&self, name: &str) -> Result<Pad> {
        self.request_pad_simple(name).with_context(|| {
            format!(
                "Unable to request pad '{name}' for element '{}' in {}",
                self.type_(),
                Location::caller()
            )
        })
    }

    #[track_caller]
    fn set_state_with_context(&self, state: State) -> Result<StateChangeSuccess> {
        self.set_state(state).with_context(|| {
            format!(
                "Unable to set state '{state:?}' for '{}' in {}",
                self.type_(),
                Location::caller()
            )
        })
    }

    #[track_caller]
    fn static_pad_with_context(&self, name: &str) -> Result<Pad> {
        self.static_pad(name).with_context(|| {
            format!(
                "Unable to get static pad '{name}' for element '{}' in {}",
                self.type_(),
                Location::caller()
            )
        })
    }

    #[track_caller]
    fn sync_state_with_parent_with_context(&self) -> Result<()> {
        self.sync_state_with_parent().with_context(|| {
            format!(
                "Unable to sync state with parent for '{}' in {}",
                self.type_(),
                Location::caller()
            )
        })
    }
}

impl<E: IsA<Element>> GstElementErrorExt for E {}

pub trait GstElementFactoryErrorExt: Sized {
    #[track_caller]
    fn make_with_name_with_context(factoryname: &str, name: Option<&str>) -> Result<Element>;
}

impl GstElementFactoryErrorExt for ElementFactory {
    #[track_caller]
    fn make_with_name_with_context(factoryname: &str, name: Option<&str>) -> Result<Element> {
        ElementFactory::make_with_name(factoryname, name).with_context(|| {
            format!(
                "Unable to make Element with factoryname '{factoryname}' and name '{name:?}' in {}",
                Location::caller()
            )
        })
    }
}

pub trait GstGhostPadErrorExt: Sized {
    #[track_caller]
    fn with_target_with_context<P: IsA<Pad> + IsA<Object>>(
        name: Option<&str>,
        target: &P,
    ) -> Result<Self>;
}

impl GstGhostPadErrorExt for GhostPad {
    #[track_caller]
    fn with_target_with_context<P: IsA<Pad> + IsA<Object>>(
        name: Option<&str>,
        target: &P,
    ) -> Result<Self> {
        let mut build = GhostPad::builder_with_target(target).with_context(|| {
            format!(
                "Unable to create a GhostPad '{:?}' with target '{}' in {}",
                name,
                target.type_(),
                Location::caller()
            )
        })?;
        if let Some(name) = name {
            build = build.name(name);
        }
        Ok(build.build())
    }
}

pub trait GstPadErrorExt: IsA<Pad> {
    #[track_caller]
    fn link_with_context(&self, dest: &impl IsA<Pad>) -> Result<PadLinkSuccess> {
        self.link(dest).with_context(|| {
            format!(
                "Unable to link pad '{}' to '{}' in {}",
                self.type_(),
                dest.type_(),
                Location::caller()
            )
        })
    }
}

impl<P: IsA<Pad>> GstPadErrorExt for P {}

#[track_caller]
pub fn parse_bin_from_description_with_context(
    bin_description: &str,
    ghost_unlinked_pads: bool,
) -> Result<Bin> {
    gstreamer::parse::bin_from_description(bin_description, ghost_unlinked_pads).with_context(|| {
            format!(
                "Unable to parse bin from description with ghost_unlinked_pad={ghost_unlinked_pads} in {}.\nbin_description:\n{bin_description}",
                Location::caller()
            )
        })
}
