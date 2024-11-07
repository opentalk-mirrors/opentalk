// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use crate::media::VideoSinks;
use ezk_image::PixelFormat;
use opentalk_compositor::{
    font::{DrawText, I420Image, Point, SimpleText},
    HEIGHT, WIDTH,
};
use parking_lot::Mutex;
use std::{sync::Arc, thread::JoinHandle};
use std::{thread, time::Duration};

const TOP_OFFSET: usize = 150;

pub(crate) struct UiController {
    data: Arc<Mutex<UiData>>,
}

impl UiController {
    pub(crate) fn set(&self, id: String, pin: String) {
        let mut data = self.data.lock();

        data.id = id;
        data.pin = pin;
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
    run: bool,
    join_handle: Option<JoinHandle<()>>,
}

pub(crate) fn spawn_ui_thread(sinks: VideoSinks) -> UiController {
    let data = Arc::new(Mutex::new(UiData {
        id: String::new(),
        pin: String::new(),
        run: true,
        join_handle: None,
    }));

    let ui_controller = UiController { data: data.clone() };

    let mut staging = vec![0u8; PixelFormat::I420.buffer_size(WIDTH, HEIGHT)];

    let dummy_id_text = SimpleText::new(100.0, "Id: 9999999999");
    let dummy_pin_text = SimpleText::new(100.0, "Pin: 9999999999");

    let text_top_offset = HEIGHT / 2 - dummy_id_text.height() as usize;

    let handle = thread::spawn(move || loop {
        let data = data.lock();

        if !data.run {
            log::debug!("ui thread exiting");
            break;
        }

        let mut image = I420Image::try_from(&mut staging, Point::new(WIDTH, HEIGHT)).unwrap();

        image.y.fill(0);
        image.u.fill(128);
        image.v.fill(128);

        let headline = SimpleText::new(128.0, "OpenTalk Call-In");
        let id_text = SimpleText::new(100.0, &format!("Id: {}", data.id));
        let pin_text = SimpleText::new(100.0, &format!("Pin: {}", data.pin));

        drop(data);

        headline.draw(
            Point::new((WIDTH - headline.width() as usize) / 2, TOP_OFFSET),
            &mut image,
        );

        id_text.draw(
            Point::new(
                (WIDTH - dummy_id_text.width() as usize) / 2
                    + ((dummy_pin_text.width() - dummy_id_text.width()) / 2.) as usize,
                text_top_offset,
            ),
            &mut image,
        );

        pin_text.draw(
            Point::new(
                (WIDTH - dummy_pin_text.width() as usize) / 2,
                text_top_offset + id_text.height() as usize + 10,
            ),
            &mut image,
        );

        for sink in sinks.blocking_lock().values_mut() {
            sink(&staging)
        }

        thread::sleep(Duration::from_millis(33));
    });

    ui_controller.data.lock().join_handle = Some(handle);
    ui_controller
}
