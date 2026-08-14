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

const MAX_TEXT_LEN: usize = 120;

fn segment_file_name(number: u64, text: &str) -> String {
    let text: String = text
        .replace(
            |c: char| {
                c.is_ascii_control()
                    || matches!(c, '/' | '\\' | ':' | '*' | '"' | '<' | '>' | '|' | '\0')
            },
            "_",
        )
        .chars()
        .take(MAX_TEXT_LEN)
        .collect();

    let text = text.trim();

    if text.is_empty() {
        format!("{number:04}.wav")
    } else {
        format!("{number:04}_{text}.wav")
    }
}

fn write_segment_wav(samples: &[f32], path: &Path) {
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

    for sample in samples {
        writer.write_sample(*sample).unwrap();
    }

    writer.flush().unwrap();
    drop(writer);

    std::fs::write(path, wav_buf).unwrap();
}

pub fn dump(segment: &SpeechSegment, text: &str, destination: &Path) {
    let number = COUNTER.fetch_add(1, Ordering::Relaxed);

    let path = destination.join(segment_file_name(number, text));

    write_segment_wav(&segment.samples, &path);
}
