// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    io::Cursor,
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
};

use hound::WavSpec;

use crate::audio_processing::voice_activity_detection::SpeechSegment;

static COUNTER: AtomicU64 = AtomicU64::new(0);

pub(crate) fn dump(segment: &SpeechSegment, text: &str, destination: &Path) {
    let number = COUNTER.fetch_add(1, Ordering::Relaxed);

    let mut wav_buf = Vec::new();

    let mut writer = hound::WavWriter::new(
        Cursor::new(&mut wav_buf),
        WavSpec {
            channels: 1,
            sample_rate: 16_000,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        },
    )
    .unwrap();

    for sample in &segment.samples {
        writer.write_sample(*sample).unwrap();
    }

    writer.flush().unwrap();
    drop(writer);

    let text = text.replace(
        |c: char| {
            c.is_ascii_control()
                || matches!(c, '/' | '\\' | ':' | '*' | '"' | '<' | '>' | '|' | '\0')
        },
        "_",
    );

    let path = destination.join(format!("{number}_{text}.wav"));

    std::fs::write(path, wav_buf).unwrap();
}
