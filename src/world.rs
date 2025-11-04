// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: EUPL-1.2
//
use std::{
    borrow::Cow,
    collections::BTreeMap,
    fs::DirEntry,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use chrono::{DateTime, Datelike as _, FixedOffset, Local, Utc};
use snafu::{OptionExt as _, ResultExt as _};
use typst::{
    Library,
    diag::{FileError, FileResult},
    foundations::{Bytes, Datetime},
    syntax::{
        FileId, Source, VirtualPath,
        package::{PackageSpec, PackageVersion},
    },
    text::{Font, FontBook},
    utils::LazyHash,
};
use typst_kit::fonts::{FontSlot, Fonts};

use crate::{
    ReportGenerationError,
    report_generation_error::{
        PackageFileGlobSnafu, PackageFileReadSnafu, PackageVersionInvalidGlobPatternSnafu,
        PackageVersionInvalidUtf8ForGlobSnafu, ReadPackageDirSnafu,
        ReadPackageVersionDirectorySnafu, ReadPackageVersionFileNameNotUtf8Snafu,
        ReadPackageVersionFileNameSnafu, ReadPackagesDirEntrySnafu, ReadPackagesDirSnafu,
        ReadPackagesNamespaceDirEntrySnafu,
    },
};

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
    pub fn new(
        text: String,
        files: BTreeMap<&Path, (Option<PackageSpec>, Cow<'static, [u8]>)>,
    ) -> Self {
        let fonts = Fonts::searcher()
            .include_system_fonts(false)
            .include_embedded_fonts(true)
            .search();

        let library = Library::builder().build();

        let files = files
            .into_iter()
            .map(|(path, (package, data))| {
                let path = if path.starts_with("/") || path.starts_with("@") {
                    VirtualPath::new(path)
                } else {
                    VirtualPath::new(Path::new("/").join(path))
                };
                log::debug!("Adding file for package {package:?}: {path:?}");
                let file_id = FileId::new(package, path);
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

    pub fn load_packages(&mut self, packages_path: &Path) -> Result<(), ReportGenerationError> {
        log::debug!("Loading typst packages from {packages_path:?}");
        let dirs = packages_path
            .read_dir()
            .with_context(|_| ReadPackagesDirSnafu {
                path: packages_path.to_path_buf(),
            })?
            .filter_map(|entry| {
                entry
                    .map(|direntry| {
                        let path = direntry.path();
                        if path.is_dir() { Some(path) } else { None }
                    })
                    .with_context(|_| ReadPackagesDirEntrySnafu {
                        packages_path: packages_path.to_path_buf(),
                    })
                    .transpose()
            });
        for dir in dirs {
            let dir = dir?;
            log::debug!("Found typst package namespace dir {dir:?}");

            let Some(file_name) = dir.file_name() else {
                log::warn!("Found path without a file name, ignoring path: {dir:?}");
                continue;
            };

            let Some(namespace) = file_name.to_str() else {
                log::warn!(
                    "Found file name which cannot be converted to UTF-8, ignoring namespace: {file_name:?}"
                );
                continue;
            };

            self.load_package_namespace(namespace, &dir)?;
        }
        Ok(())
    }

    fn load_package_namespace(
        &mut self,
        namespace: &str,
        namespace_path: &Path,
    ) -> Result<(), ReportGenerationError> {
        log::debug!(
            "Loading typst packages from namespace {namespace:?} in dir {namespace_path:?}"
        );
        let dirs = namespace_path
            .read_dir()
            .with_context(|_| ReadPackagesDirSnafu {
                path: namespace_path.to_path_buf(),
            })?
            .filter_map(|entry| {
                entry
                    .map(|direntry| {
                        let path = direntry.path();
                        if path.is_dir() { Some(path) } else { None }
                    })
                    .with_context(|_| ReadPackagesNamespaceDirEntrySnafu {
                        namespace_path: namespace_path.to_path_buf(),
                    })
                    .transpose()
            });
        for dir in dirs {
            let dir = dir?;
            log::debug!("Found package in namespace dir {dir:?}");

            let Some(file_name) = dir.file_name() else {
                log::warn!("Found path without a file name, ignoring path: {dir:?}");
                continue;
            };

            let Some(package) = file_name.to_str() else {
                log::warn!(
                    "Found file name which cannot be converted to UTF-8, ignoring package: {file_name:?}"
                );
                continue;
            };

            self.load_package(namespace, package, &dir)?;
        }
        Ok(())
    }

    fn load_package(
        &mut self,
        namespace: &str,
        package: &str,
        package_path: &Path,
    ) -> Result<(), ReportGenerationError> {
        log::debug!("Loading package {package:?} versions from {package_path:?}");

        fn load_path_buf_and_package_spec(
            entry: &DirEntry,
            namespace: &str,
            package: &str,
        ) -> Result<Option<(PathBuf, PackageSpec)>, ReportGenerationError> {
            let path = entry.path();
            if path.is_dir() {
                let version = path
                    .file_name()
                    .with_context(|| ReadPackageVersionFileNameSnafu {
                        package_version_path: path.to_path_buf(),
                    })?
                    .to_str()
                    .with_context(|| ReadPackageVersionFileNameNotUtf8Snafu {
                        package_version_path: path.to_path_buf(),
                    })?;
                let Ok(version) = version.parse::<PackageVersion>() else {
                    log::debug!("Directory {path:?} is not a package version number, ignoring");
                    return Ok(None);
                };
                log::debug!("Found directory {path:?} for package version {version}");
                Ok(Some((
                    path,
                    PackageSpec {
                        namespace: namespace.into(),
                        name: package.into(),
                        version,
                    },
                )))
            } else {
                Ok(None)
            }
        }

        let package_entries = package_path
            .read_dir()
            .with_context(|_| ReadPackageDirSnafu {
                package_path: package_path.to_path_buf(),
            })?
            .filter_map(|entry| {
                entry
                    .with_context(|_| ReadPackageVersionDirectorySnafu {
                        package_version_path: package_path.to_path_buf(),
                    })
                    .and_then(|entry| load_path_buf_and_package_spec(&entry, namespace, package))
                    .transpose()
            });

        for package_info in package_entries {
            let (path, package_spec) = package_info?;
            self.load_package_version(&path, package_spec)?;
        }
        Ok(())
    }

    fn load_package_version(
        &mut self,
        path: &Path,
        package_spec: PackageSpec,
    ) -> Result<(), ReportGenerationError> {
        log::debug!(
            "Loading package {} from namespace {} version {} from {path:?}",
            package_spec.name,
            package_spec.namespace,
            package_spec.version
        );
        let glob_path = path
            .to_str()
            .with_context(|| PackageVersionInvalidUtf8ForGlobSnafu {
                package_version_path: path.to_path_buf(),
            })?;

        let glob_pattern = format!("{glob_path}/**/*");
        log::debug!("Globbing package version path with pattern {glob_pattern:?}");
        let file_paths = glob::glob(&glob_pattern)
            .with_context(|_| PackageVersionInvalidGlobPatternSnafu { glob_pattern })?
            .map(|entry| {
                entry.with_context(|e| PackageFileGlobSnafu {
                    path: e.path().to_path_buf(),
                })
            });
        for file_path in file_paths {
            let file_path = file_path?;

            if file_path.is_file() {
                log::trace!("Loading file {file_path:?} for {package_spec:?}");
                let relative = file_path.strip_prefix(path).expect(
                    "Couldn't strip prefix from file path, although it was created by globbing this exact prefix");
                log::trace!("Relative path is {relative:?}");
                let data = std::fs::read(&file_path).with_context(|_| PackageFileReadSnafu {
                    path: file_path.to_path_buf(),
                })?;
                let file_id = FileId::new(
                    Some(package_spec.clone()),
                    VirtualPath::new(Path::new("/").join(relative)),
                );
                let _ = self.files.insert(file_id, Bytes::new(data));
            }
        }
        Ok(())
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
            self.files
                .get(&id)
                .map(|b| Source::new(id, String::from_utf8_lossy(b).to_string()))
                .ok_or(FileError::NotSource)
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
