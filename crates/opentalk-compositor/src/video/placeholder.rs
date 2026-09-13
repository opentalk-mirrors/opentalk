// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::I420_COLOR;
use anyhow::{Context, Result};
use ezk_image::{Image, ImageRef, PixelFormat};
use image::{imageops::FilterType, DynamicImage, ImageBuffer, Rgba};
use livekit::webrtc::prelude::I420Buffer;

const PLACEHOLDER_WIDTH: u32 = 640;
const PLACEHOLDER_HEIGHT: u32 = 360;

const AVATAR_SIZE: u32 = 192;

pub(crate) const PLACEHOLDER_BACKGROUND_COLOR: [u8; 4] = [0x13, 0x26, 0x2d, 0xFF];

pub(crate) fn avatar_to_placeholder(avatar: &DynamicImage) -> Result<I420Buffer> {
    let avatar = avatar.resize(AVATAR_SIZE, AVATAR_SIZE, FilterType::Lanczos3);
    let avatar_rgba = avatar.to_rgba8();

    let mut base_image_rgba =
        ImageBuffer::<Rgba<u8>, _>::new(PLACEHOLDER_WIDTH, PLACEHOLDER_HEIGHT);
    for pixel in base_image_rgba.pixels_mut() {
        pixel.0 = PLACEHOLDER_BACKGROUND_COLOR;
    }

    image::imageops::overlay(
        &mut base_image_rgba,
        &avatar_rgba,
        ((PLACEHOLDER_WIDTH - AVATAR_SIZE) / 2).into(),
        ((PLACEHOLDER_HEIGHT - AVATAR_SIZE) / 2).into(),
    );

    let base_image = ezk_image::Image::from_buffer(
        PixelFormat::RGBA,
        base_image_rgba.as_raw(),
        None,
        base_image_rgba.width() as usize,
        base_image_rgba.height() as usize,
        I420_COLOR,
    )
    .context("Failed to create Image from placeholder rgba")?;

    convert_to_i420_buffer(&base_image)
}

pub(crate) fn load_placeholder_image() -> Result<I420Buffer> {
    let placeholder_image = image::load_from_memory(include_bytes!("../../assets/placeholder.png"))
        .context("Failed to load placeholder.png")?;

    let rgb_placeholder_image = Image::from_buffer(
        PixelFormat::RGB,
        placeholder_image.to_rgb8().into_raw(),
        None,
        placeholder_image.width() as usize,
        placeholder_image.height() as usize,
        I420_COLOR,
    )
    .context("Failed to create Image from placeholder.png asset")?;

    convert_to_i420_buffer(&rgb_placeholder_image)
}

fn convert_to_i420_buffer(image: &dyn ImageRef) -> Result<I420Buffer> {
    let mut i420_buffer = I420Buffer::new(image.width() as u32, image.height() as u32);

    let (y_stride, u_stride, v_stride) = i420_buffer.strides();
    let (y, u, v) = i420_buffer.data_mut();

    let mut i420_image = Image::from_planes(
        PixelFormat::I420,
        vec![y, u, v],
        Some(vec![
            y_stride as usize,
            u_stride as usize,
            v_stride as usize,
        ]),
        image.width(),
        image.height(),
        I420_COLOR,
    )
    .context("Failed to create Image from livekit's I420Buffer")?;

    ezk_image::convert_multi_thread(image, &mut i420_image)?;

    Ok(i420_buffer)
}
