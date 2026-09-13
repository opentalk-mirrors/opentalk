// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use anyhow::{Context, Result};
use ezk_image::{ImageMut, ImageRef, PixelFormat};
use image::math::Rect;
use livekit::webrtc::prelude::{I420Buffer, VideoBuffer as _};

use crate::I420_COLOR;

pub(crate) fn blend_yuv(target: u8, alpha: f32, value: f32) -> u8 {
    ((1.0 - alpha) * f32::from(target) + (value * alpha)) as u8
}

#[derive(Clone, Copy)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}

impl<T> Point<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl From<Point<usize>> for Rect {
    fn from(val: Point<usize>) -> Self {
        Rect {
            x: val.x as u32,
            y: val.y as u32,
            width: 0,
            height: 0,
        }
    }
}

pub struct I420Image<'a> {
    pub resolution: Point<usize>,

    pub y: &'a mut [u8],
    pub u: &'a mut [u8],
    pub v: &'a mut [u8],
}

impl<'a> I420Image<'a> {
    /// # Errors
    ///
    /// If the img buffer is not a valid I420 image
    pub fn try_from(img: &'a mut [u8], resolution: Point<usize>) -> Result<I420Image<'a>> {
        let (yv, tmp) = img
            .split_at_mut_checked(resolution.x * resolution.y)
            .context("img smaller than pixel size")?;
        let (uv, vv) = tmp.split_at_mut((resolution.x * resolution.y) / 4);

        Ok(Self {
            resolution,

            y: yv,
            u: uv,
            v: vv,
        })
    }

    #[track_caller]
    pub(crate) fn get_luma(&mut self, x: usize, y: usize) -> &mut u8 {
        let luma_index = y * self.resolution.x + x;
        &mut self.y[luma_index]
    }

    #[track_caller]
    pub(crate) fn get_chroma_u(&mut self, x: usize, y: usize) -> &mut u8 {
        let u_index = (y / 2) * (self.resolution.x / 2) + (x / 2);
        &mut self.u[u_index]
    }

    #[track_caller]
    pub(crate) fn get_chroma_v(&mut self, x: usize, y: usize) -> &mut u8 {
        let v_index = (y / 2) * (self.resolution.x / 2) + (x / 2);
        &mut self.v[v_index]
    }

    #[track_caller]
    pub(crate) fn get_luma_range(&mut self, x: usize, y: usize, amount: usize) -> &mut [u8] {
        let start_y_index = y * self.resolution.x + x;
        let end_y_index = start_y_index + amount;
        &mut self.y[start_y_index..end_y_index]
    }

    #[track_caller]
    pub(crate) fn get_chroma_u_range(&mut self, x: usize, y: usize, amount: usize) -> &mut [u8] {
        let start_u_index = (y / 2) * (self.resolution.x / 2) + (x / 2);
        let end_u_index = start_u_index + amount / 2;
        &mut self.u[start_u_index..end_u_index]
    }

    #[track_caller]
    pub(crate) fn get_chroma_v_range(&mut self, x: usize, y: usize, amount: usize) -> &mut [u8] {
        let start_v_index = (y / 2) * (self.resolution.x / 2) + (x / 2);
        let end_v_index = start_v_index + amount / 2;
        &mut self.v[start_v_index..end_v_index]
    }
}

unsafe impl ImageRef for I420Image<'_> {
    fn format(&self) -> PixelFormat {
        PixelFormat::I420
    }

    fn width(&self) -> usize {
        self.resolution.x
    }

    fn height(&self) -> usize {
        self.resolution.y
    }

    fn planes(&self) -> Box<dyn Iterator<Item = (&[u8], usize)> + '_> {
        let strides = self.format().packed_strides(self.resolution.x);
        Box::new([&*self.y, &*self.u, &*self.v].into_iter().zip(strides))
    }

    fn color(&self) -> ezk_image::ColorInfo {
        I420_COLOR
    }
}

unsafe impl ImageMut for I420Image<'_> {
    fn planes_mut(&mut self) -> Box<dyn Iterator<Item = (&mut [u8], usize)> + '_> {
        let strides = self.format().packed_strides(self.resolution.x);
        Box::new([&mut *self.y, self.u, self.v].into_iter().zip(strides))
    }
}

/// Wrapper over livekit's I420Buffer implementing ezk-image's ImageRef
pub(crate) struct I420BufferImageRef<'a>(pub(crate) &'a I420Buffer);

unsafe impl ImageRef for I420BufferImageRef<'_> {
    fn format(&self) -> PixelFormat {
        PixelFormat::I420
    }

    fn width(&self) -> usize {
        self.0.width() as usize
    }

    fn height(&self) -> usize {
        self.0.height() as usize
    }

    fn planes(&self) -> Box<dyn Iterator<Item = (&[u8], usize)> + '_> {
        let (y_stride, u_stride, v_stride) = self.0.strides();
        let (y, u, v) = self.0.data();

        Box::new(
            [
                (y, y_stride as usize),
                (u, u_stride as usize),
                (v, v_stride as usize),
            ]
            .into_iter(),
        )
    }

    fn color(&self) -> ezk_image::ColorInfo {
        I420_COLOR
    }
}
