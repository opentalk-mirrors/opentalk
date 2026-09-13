// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::{Context, Result};
use ezk_image::{Image, PixelFormat};
use resvg::tiny_skia::{Color, Pixmap};
use resvg::usvg::{Options, Transform, Tree};

use crate::video::placeholder::PLACEHOLDER_BACKGROUND_COLOR;
use crate::I420_COLOR;

/// Render a SVG file to a I420 image from source
pub(crate) fn load(source: &[u8]) -> Result<Image<Vec<u8>>> {
    // SCALE = Quality (1x scale is 24x24 px icon) (currently hardcoded for mic-off/cam-off icons)
    const SCALE: f32 = 1.5;
    const SIZE: u32 = (24.0 * SCALE) as u32;

    let tree =
        Tree::from_data(source, &Options::default()).context("Failed to load SVG from source")?;

    let mut pixmap = Pixmap::new(SIZE, SIZE).context("Failed to create pixmap")?;

    // Fill the pixmap with the placeholder background color
    let [r, g, b, a] = PLACEHOLDER_BACKGROUND_COLOR;
    pixmap.fill(Color::from_rgba8(r, g, b, a));

    // Render the svg to the pixmap
    resvg::render(
        &tree,
        Transform::from_scale(SCALE, SCALE),
        &mut pixmap.as_mut(),
    );

    // Convert the RGBA pixmap to I420
    let pixmap = Image::from_buffer(
        PixelFormat::RGBA,
        pixmap.data(),
        None,
        SIZE as usize,
        SIZE as usize,
        I420_COLOR,
    )
    .context("Failed to create Image from pixmap")?;

    let mut i420_icon = Image::blank(PixelFormat::I420, SIZE as usize, SIZE as usize, I420_COLOR);

    ezk_image::convert(&pixmap, &mut i420_icon)
        .context("Failed to convert SVG icon from RGBA to I420")?;

    Ok(i420_icon)
}
