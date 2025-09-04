// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2
//
use std::{borrow::Cow, collections::BTreeMap, path::Path, sync::LazyLock};

use chrono::{DateTime, Datelike as _, FixedOffset, Local, Utc};
use typst::{
    Library,
    diag::{FileError, FileResult},
    foundations::{Bytes, Datetime},
    syntax::{FileId, Source, VirtualPath},
    text::{Font, FontBook},
    utils::LazyHash,
};
use typst_kit::fonts::{FontSlot, Fonts};

pub(crate) static MAIN_ID: LazyLock<FileId> =
    LazyLock::new(|| FileId::new_fake(VirtualPath::new("<main>")));

pub struct World {
    main: Source,
    fonts: Vec<FontSlot>,
    now: DateTime<Utc>,
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    files: BTreeMap<FileId, Bytes>,
}

impl World {
    pub fn new(text: String, files: BTreeMap<&Path, Cow<'static, [u8]>>) -> Self {
        let fonts = Fonts::searcher()
            .include_system_fonts(false)
            .include_embedded_fonts(true)
            .search();

        let library = Library::builder().build();

        let files = files
            .into_iter()
            .map(|(path, data)| {
                let path = if path.starts_with("/") {
                    VirtualPath::new(path)
                } else {
                    VirtualPath::new(Path::new("/").join(path))
                };
                let file_id = FileId::new(None, path);
                (file_id, Bytes::new(data))
            })
            .collect();

        Self {
            main: Source::new(*MAIN_ID, text),
            fonts: fonts.fonts,
            now: Utc::now(),
            library: LazyHash::new(library),
            book: LazyHash::new(fonts.book),
            files,
        }
    }
}

impl typst::World for World {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    fn main(&self) -> FileId {
        *MAIN_ID
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == *MAIN_ID {
            Ok(self.main.clone())
        } else {
            Err(FileError::NotSource)
        }
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        if id == *MAIN_ID {
            Ok(Bytes::new(self.main.text().to_string()))
        } else {
            self.files.get(&id).cloned().ok_or(FileError::NotSource)
        }
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).and_then(FontSlot::get)
    }

    fn today(&self, offset: Option<i64>) -> Option<Datetime> {
        let with_offset = match offset {
            None => self.now.with_timezone(&Local).fixed_offset(),
            Some(hours) => {
                let seconds = i32::try_from(hours).ok()?.checked_mul(3600)?;
                self.now.with_timezone(&FixedOffset::east_opt(seconds)?)
            }
        };
        Datetime::from_ymd(
            with_offset.year(),
            with_offset.month().try_into().ok()?,
            with_offset.day().try_into().ok()?,
        )
    }
}
