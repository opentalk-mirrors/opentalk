// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

//! Functions for debugging..

use glib::object::Cast;
use gstreamer::{
    prelude::{GstBinExtManual, GstObjectExt as _},
    DebugGraphDetails,
};

/// Pipeline DOT debugging parameters
pub struct Params {
    /// Graphics details like described in gstreamer
    pub details: DebugGraphDetails,
    /// Use an index prefix for the output files
    pub index: bool,
}

impl Params {
    /// all details
    #[must_use]
    pub const fn all() -> Self {
        Self {
            details: DebugGraphDetails::ALL,
            index: true,
        }
    }
    /// show states
    #[must_use]
    pub const fn states() -> Self {
        Self {
            details: DebugGraphDetails::STATES,
            index: true,
        }
    }
}

impl Default for Params {
    fn default() -> Self {
        Self {
            details: DebugGraphDetails::ALL,
            index: true,
        }
    }
}

/// Make a DOT file of the given element if log level is `debug`.
///
/// # Arguments
///
/// - `element`: Element in the pipeline which shall be generated a DOT file from.
///
pub fn debug_dot(
    element: &impl glib::object::IsA<gstreamer::Element>,
    filename_without_extension: &str,
) {
    if log::max_level() >= log::Level::Debug {
        dot(element, filename_without_extension);
    }
}

/// Make a DOT file of the given element with a counting index and default parameters.
///
/// # Arguments
///
/// - `element`: Element in the pipeline which shall be generated a DOT file from.
///
pub fn dot(element: &impl glib::object::IsA<gstreamer::Element>, filename_without_extension: &str) {
    dot_ext(element, filename_without_extension, &Params::default());
}

/// Make a DOT file of the given element with a counting index and the given parameters.
///
/// # Arguments
///
/// - `element`: Element in the pipeline which shall be generated a DOT file from.
pub fn dot_ext(
    element: &impl glib::object::IsA<gstreamer::Element>,
    filename_without_extension: &str,
    params: &Params,
) {
    // count calls
    use std::sync::atomic::{AtomicUsize, Ordering};
    static COUNT: AtomicUsize = AtomicUsize::new(0);

    // check if env var `GST_DEBUG_DUMP_DOT_DIR` has been set properly
    let Ok(path) = std::env::var("GST_DEBUG_DUMP_DOT_DIR") else {
        return;
    };

    // create output directory if not exist
    if let Err(e) = std::fs::create_dir_all(&path) {
        error!(
            "Generation of dot file failed: can not create dir from GST_DEBUG_DUMP_DOT_DIR: {e:?}"
        );
        return;
    }

    let Ok(object) = element.clone().dynamic_cast::<gstreamer::Object>() else {
        error!(
            "Generation of dot file failed: unable to dynamic cast the `element` to 'gstreamer::Object'"
        );
        return;
    };

    // recursion to top parent
    if let Some(parent) = object.parent() {
        let Ok(element) = parent.dynamic_cast::<gstreamer::Element>() else {
            error!("Generation of dot file failed: unable to dynamic cast the `object` to 'gstreamer::Element'");
            return;
        };

        dot(&element, filename_without_extension);

        return;
    }

    // count file name index if configured
    let name = if params.index {
        let n = COUNT.fetch_add(1, Ordering::SeqCst);
        let r = format!("{n}-{filename_without_extension}");
        r
    } else {
        filename_without_extension.to_string()
    };

    // generate DOT file
    info!("GENERATING DOT FILE: '{path}/{name}.dot'");

    let Ok(bin) = Cast::dynamic_cast::<gstreamer::Bin>(element.clone()) else {
        error!("Generation of dot file failed: unable to cast element to bin");
        return;
    };
    bin.debug_to_dot_file(params.details, name);
}

/// Create a name (and parent name as suffix) from given gstreamer object.
///
/// # Arguments
///
/// - `object`: Object to return name from.
///
pub fn name(object: &impl glib::object::IsA<gstreamer::Object>) -> glib::GString {
    if let Some(parent) = object.parent() {
        format!("{}.{}", name(&parent), object.name()).into()
    } else {
        object.name()
    }
}
