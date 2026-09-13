// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::image::{blend_yuv, I420Image, Point};
use ab_glyph::{point, Font, FontRef, Glyph, PxScale, ScaleFont};
use std::sync::OnceLock;

pub(crate) fn create_font() -> &'static FontRef<'static> {
    static FONT: OnceLock<FontRef<'static>> = OnceLock::new();

    FONT.get_or_init(|| {
        FontRef::try_from_slice(include_bytes!(
            "../assets/opentalk-font/regular/opentalk-regular.ttf"
        ))
        .expect("font could not be loaded")
    })
}

pub trait DrawText {
    fn draw(&self, location: Point<usize>, image: &mut I420Image<'_>);
}

pub struct SimpleText {
    font_info: &'static FontRef<'static>,
    glyphs: Vec<Glyph>,
    scale: PxScale,
}

impl SimpleText {
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn new(text_scale: f32, text: &str) -> Self {
        let font_info = create_font();

        let scaled = font_info.as_scaled(text_scale);

        // let font_padding: f32 = 0.2; // + outline;
        let mut caret = point(0.0, scaled.ascent());
        let mut last_glyph: Option<Glyph> = None;
        let glyphs = text
            .chars()
            .map(|c| {
                let mut glyph = scaled.scaled_glyph(c);
                if let Some(previous) = last_glyph.take() {
                    caret.x += scaled.kern(previous.id, glyph.id);
                }
                glyph.position = caret;

                last_glyph = Some(glyph.clone());
                caret.x += scaled.h_advance(glyph.id);
                glyph
            })
            .collect();

        Self {
            font_info,
            glyphs,
            scale: PxScale::from(text_scale),
        }
    }

    #[must_use]
    pub fn width(&self) -> f32 {
        let min_x = self
            .glyphs
            .first()
            .map(|g| g.position.x)
            .unwrap_or_default();
        let max_x = self
            .glyphs
            .last()
            .map(|g| g.position.x + g.scale.x)
            .unwrap_or_default();

        max_x - min_x
    }

    #[must_use]
    pub fn height(&self) -> f32 {
        let ascent = self.font_info.as_scaled(self.scale.y).ascent();
        let descent = self.font_info.as_scaled(self.scale.y).descent();

        ascent - descent
    }
}

impl DrawText for SimpleText {
    fn draw(&self, location: Point<usize>, image: &mut I420Image<'_>) {
        for g in &self.glyphs {
            let Some(g) = self.font_info.outline_glyph(g.clone()) else {
                continue;
            };

            g.draw(|x, y, v| {
                let x = x as usize;
                let y = y as usize;

                let (x, y) = (
                    x + location.x + g.px_bounds().min.x as usize,
                    y + location.y + g.px_bounds().min.y as usize,
                );

                if x > image.resolution.x {
                    return;
                }

                if y > image.resolution.y {
                    return;
                }

                let current_luma = image.get_luma(x, y);
                *current_luma = blend_yuv(*current_luma, v, 255.0);

                let current_u = image.get_chroma_u(x, y);
                *current_u = blend_yuv(*current_u, v, 128.0);

                let current_v = image.get_chroma_v(x, y);
                *current_v = blend_yuv(*current_v, v, 128.0);
            });
        }
    }
}

pub(crate) struct TextBox {
    simple_text: SimpleText,
}

impl TextBox {
    pub fn new(simple_text: SimpleText) -> Self {
        Self { simple_text }
    }

    fn width_padding(&self) -> f32 {
        self.simple_text
            .font_info
            .as_scaled(self.simple_text.scale.x)
            .scale
            .x
    }

    fn height_padding(&self) -> f32 {
        self.simple_text
            .font_info
            .as_scaled(self.simple_text.scale.y)
            .scale
            .y
            / 2.
    }

    pub fn width(&self) -> f32 {
        self.simple_text.width() + self.width_padding()
    }

    pub fn height(&self) -> f32 {
        self.simple_text.height() + self.height_padding()
    }
}

impl DrawText for TextBox {
    fn draw(&self, location: Point<usize>, image: &mut I420Image<'_>) {
        for x in location.x..location.x + self.width() as usize {
            for y in location.y..location.y + self.height() as usize {
                let v = 0.8; // the "alpha"

                let current_luma = image.get_luma(x, y);
                *current_luma = blend_yuv(*current_luma, v, 50.0);
            }
        }

        self.simple_text.draw(
            Point::new(
                location.x + self.width_padding() as usize / 2,
                location.y + self.height_padding() as usize / 2,
            ),
            image,
        );
    }
}
