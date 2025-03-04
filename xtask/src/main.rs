// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2

use std::{
    env::{self},
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

macro_rules! text {
    ($($ident:ident: $text_de:expr, $text_en:expr;)*) => {
        pub(crate) static TEXT: &[[&str; 3]] = &[
            $([stringify!($ident), $text_de, $text_en]),*
        ];
    };
}

mod text;

fn main() -> Result<(), ()> {
    let piper = env::var("PIPER_BIN").expect("Please set the PIPER_BIN environment variable");
    let model_de = env::var("MODEL_DE").expect("MODEL_DE environment variable must be set");
    let model_en = env::var("MODEL_EN").expect("MODEL_EN environment variable must be set");

    let temp_dir = temp_dir::TempDir::new().unwrap();

    let filter = env::args().nth(1);

    for (i, [ident, text_de, text_en]) in text::TEXT.iter().enumerate() {
        if let Some(filter) = &filter {
            if *ident != filter.trim() {
                continue;
            }
        }

        println!("[{:2}/{}] Generate {ident}", i + 1, text::TEXT.len(),);

        generate_track(
            &piper,
            &model_de,
            &temp_dir.path().join(format!("{ident}_de")),
            text_de,
            &format!("audio/de/{ident}.wav"),
        )?;

        generate_track(
            &piper,
            &model_en,
            &temp_dir.path().join(format!("{ident}_en")),
            text_en,
            &format!("audio/en/{ident}.wav"),
        )?;
    }

    println!("Track macro input (src/media/track.rs):");
    for [ident, ..] in text::TEXT {
        println!("\t{ident:?}, {};", heck::AsUpperCamelCase(ident));
    }

    Ok(())
}

fn generate_track(piper: &str, model: &str, tmp: &Path, text: &str, dst: &str) -> Result<(), ()> {
    let mut piper_cmd = Command::new(piper)
        .arg("-m")
        .arg(model)
        .arg("--length_scale")
        .arg("1.1")
        .arg("--sentence_silence")
        .arg("0.17")
        .arg("--noise_w")
        .arg("0.8")
        .arg("--noise_scale")
        .arg("0.667")
        .arg("-f")
        .arg(tmp)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();

    let mut stdin = piper_cmd.stdin.take().unwrap();

    stdin.write_all(text.as_bytes()).unwrap();
    drop(stdin);

    let output = piper_cmd.wait_with_output().unwrap();

    if !output.status.success() {
        println!(
            "piper exited with following output:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );

        return Err(());
    }

    let ffmpeg_cmd = Command::new("ffmpeg")
        .arg("-y")
        .arg("-i")
        .arg(tmp)
        .arg("-ar")
        .arg("48000")
        .arg("-sample_fmt")
        .arg("s16")
        .arg("-ac")
        .arg("2")
        .arg(dst)
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let output = ffmpeg_cmd.wait_with_output().unwrap();

    if !output.status.success() {
        println!(
            "ffmpeg exited with following output:\n{}",
            String::from_utf8_lossy(&output.stderr)
        );
        return Err(());
    }

    Ok(())
}
