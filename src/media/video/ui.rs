// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::media::ToCallerVideoSinks;
use ezk_image::PixelFormat;
use opentalk_compositor::{
    HEIGHT, WIDTH,
    font::{DrawText, SimpleText},
    image::{I420Image, Point},
};
use parking_lot::Mutex;
use std::{sync::Arc, thread::JoinHandle};
use std::{thread, time::Duration};

const HEADLINE_TOP_OFFSET: usize = 150;

pub(crate) struct UiController {
    data: Arc<Mutex<UiData>>,
}

impl UiController {
    pub(crate) fn set(&self, id: String, pin: String) {
        let mut data = self.data.lock();

        data.id = id;
        data.pin = pin;
    }

    pub(crate) fn set_waiting_room(&self, waiting_room: bool) {
        self.data.lock().waiting_room = waiting_room;
    }

    pub(crate) fn set_target_fps(&self, target_fps: u32) {
        self.data.lock().target_fps = target_fps;
    }

    pub(crate) fn stop(&self) {
        let mut data = self.data.lock();
        if let Some(join_handle) = data.join_handle.take() {
            data.run = false;
            drop(data);
            let _ = join_handle.join();
        }
    }
}

impl Drop for UiController {
    fn drop(&mut self) {
        self.stop();
    }
}

struct UiData {
    id: String,
    pin: String,
    waiting_room: bool,
    run: bool,
    target_fps: u32,
    join_handle: Option<JoinHandle<()>>,
}

pub(crate) fn spawn_ui_thread(sinks: ToCallerVideoSinks, waiting_room: bool) -> UiController {
    let data = Arc::new(Mutex::new(UiData {
        id: String::new(),
        pin: String::new(),
        waiting_room,
        run: true,
        target_fps: 1,
        join_handle: None,
    }));

    let ui_controller = UiController { data: data.clone() };

    let mut staging = vec![0u8; PixelFormat::I420.buffer_size(WIDTH, HEIGHT)];

    let dummy_id_text = SimpleText::new(100.0, "Id: 9999999999");
    let dummy_pin_text = SimpleText::new(100.0, "Pin: 9999999999");

    #[expect(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    let text_top_offset = HEIGHT / 2 - dummy_id_text.height() as usize;

    let handle = thread::spawn(move || {
        loop {
            let data = data.lock();
            #[expect(clippy::cast_precision_loss)]
            let sleep_delta = Duration::from_secs_f32(1.0 / data.target_fps as f32);

            if !data.run {
                log::debug!("ui thread exiting");
                break;
            }

            let mut image = I420Image::try_from(&mut staging, Point::new(WIDTH, HEIGHT)).unwrap();

            image.y.fill(0);
            image.u.fill(128);
            image.v.fill(128);

            if data.waiting_room {
                let in_waiting_room = SimpleText::new(128.0, "In Waiting-Room");

                #[expect(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
                let y = (HEIGHT / 2) - (in_waiting_room.height() / 2.0) as usize;
                #[expect(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
                let x = (WIDTH / 2) - (in_waiting_room.width() / 2.0) as usize;

                in_waiting_room.draw(Point::new(x, y), &mut image);
            } else {
                let headline = SimpleText::new(128.0, "OpenTalk Call-In");
                let id_text = SimpleText::new(100.0, &format!("Id: {}", data.id));
                let pin_text = SimpleText::new(100.0, &format!("Pin: {}", data.pin));

                drop(data);

                #[expect(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
                headline.draw(
                    Point::new(
                        // Align the Headline in the center
                        (WIDTH - headline.width() as usize) / 2,
                        HEADLINE_TOP_OFFSET,
                    ),
                    &mut image,
                );

                {
                    // Draw "Id: XXXXXXXXX"
                    //
                    // First align the text in the center using the dummy text
                    #[expect(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
                    let align_center = (WIDTH - dummy_id_text.width() as usize) / 2;
                    // Then to align the colon of both "Id:" and "Pin:" shift the "Id:" text right,
                    // by adding the half of their width difference to x
                    #[expect(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
                    let align_colons =
                        ((dummy_pin_text.width() - dummy_id_text.width()) / 2.) as usize;

                    id_text.draw(
                        Point::new(align_center + align_colons, text_top_offset),
                        &mut image,
                    );
                }

                #[expect(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
                pin_text.draw(
                    Point::new(
                        // Align the text in the exact center using the width of the dummy pin text
                        (WIDTH - dummy_pin_text.width() as usize) / 2,
                        // Align the text 10px below the "Id:" text
                        text_top_offset + id_text.height() as usize + 10,
                    ),
                    &mut image,
                );
            }

            for sink in sinks.blocking_lock().values_mut() {
                sink(&staging);
            }

            thread::sleep(sleep_delta);
        }
    });

    ui_controller.data.lock().join_handle = Some(handle);
    ui_controller
}
