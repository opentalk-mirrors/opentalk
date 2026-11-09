// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use glib::object::ObjectExt;
use gstreamer::ElementFactory;

/// Find GStreamer encoder element for device path
///
/// Elements for the renderD128 device are named vah264lpenc for low power and vah264enc for normal
/// power
///
/// For all other devices (e.g. renderD129) the name contains the device name, like this:
/// vah264renderD129lpenc
pub(crate) fn find_va_encoder(codec: &str, device: Option<&Path>) -> Result<String> {
    let device = device
        .map(|device| {
            fs::canonicalize(device)
                .with_context(|| format!("Failed to resolve VA device '{}'", device.display()))
        })
        .transpose()?;

    let mut prefixes = vec!["va".to_owned()];
    if let Some(render_node) = device.as_deref().and_then(Path::file_name) {
        prefixes.push(format!("va{}", render_node.to_string_lossy()));
    }

    for suffix in ["lpenc", "enc"] {
        for prefix in &prefixes {
            let name = format!("{prefix}{codec}{suffix}");
            let Some(factory) = ElementFactory::find(&name) else {
                continue;
            };

            if let Some(device) = &device {
                // Make sure the encoder uses the requested device
                let encoder = factory
                    .create()
                    .build()
                    .with_context(|| format!("Failed to create '{name}'"))?;

                let device_path = encoder
                    .property::<Option<String>>("device-path")
                    .map(PathBuf::from);

                if device_path.as_ref() != Some(device) {
                    continue;
                }
            }

            return Ok(name);
        }
    }

    match device {
        Some(device) => bail!(
            "No VA {codec} encoder found for device '{}'",
            device.display()
        ),
        None => bail!("No VA {codec} encoder found"),
    }
}
