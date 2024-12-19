// SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//! # opentalk-version
//!
//! Collect a set of build information and make it available for printing.
//!
//! ## Example usage:
//!
//! Add the following to your `build.rs`
//!
//! ```rust
//! fn main() -> anyhow::Result<()> {
//!     opentalk_version::collect_build_information()
//! }
//! ```
//!
//! Once this is added, you can access the build information in your crate like this:
//!
//! ```ignore
//! // This adds a `build_info` module which contains a `BuildInfo` struct that
//! // contains all collected build information.
//! opentalk_version::build_info!();
//!
//! pub(super) fn print_version() {
//!     println!("{}", build_info::BuildInfo::new())
//! }
//!
//! ```

use anyhow::Context as _;
use vergen::{BuildBuilder, CargoBuilder, Emitter, RustcBuilder};
use vergen_gix::GixBuilder;

/// This macro will expand to a module `build_info` which contains a struct `BuildInfo`.
/// The struct can be used to access build information.
///
/// # Note
///
/// Make sure to call [`collect_build_information`] in your [build script](https://doc.rust-lang.org/cargo/reference/build-scripts.html).
#[macro_export]
macro_rules! build_info {
    () => {
        pub(crate) mod build_info {
            fn profile_human() -> &'static str {
                let profile = env!("VERGEN_CARGO_OPT_LEVEL");

                // Copied from https://doc.rust-lang.org/cargo/reference/profiles.html#opt-level
                match profile {
                    "0" => "0, no optimizations",
                    "1" => "1, basic optimizations",
                    "2" => "2, some optimizations",
                    "3" => "3, all optimizations",
                    "s" => "'s', optimize for binary size",
                    "z" => "'z', optimize for binary size, but also turn off loop vectorization.",
                    profile => profile,
                }
            }

            /// Build information about the crate.
            #[derive(Debug, Clone, Copy)]
            pub struct BuildInfo {
                build_timestamp: &'static str,
                build_version: &'static str,
                commit_sha: Option<&'static str>,
                commit_dirty: Option<&'static str>,
                commit_date: Option<&'static str>,
                commit_branch: Option<&'static str>,
                rustc_version: &'static str,
                rustc_channel: &'static str,
                rustc_host_triple: &'static str,
                cargo_target_triple: &'static str,
                cargo_profile: &'static str,
            }

            impl BuildInfo {
                /// Returns a struct which contains build information.
                pub fn new() -> Self {
                    Self {
                        build_timestamp: env!("VERGEN_BUILD_TIMESTAMP"),
                        build_version: env!("CARGO_PKG_VERSION"),
                        commit_sha: option_env!("VERGEN_GIT_SHA"),
                        commit_dirty: option_env!("VERGEN_GIT_DIRTY"),
                        commit_date: option_env!("VERGEN_GIT_COMMIT_TIMESTAMP"),
                        commit_branch: option_env!("VERGEN_GIT_BRANCH"),
                        rustc_version: env!("VERGEN_RUSTC_SEMVER"),
                        rustc_channel: env!("VERGEN_RUSTC_CHANNEL"),
                        rustc_host_triple: env!("VERGEN_RUSTC_HOST_TRIPLE"),
                        cargo_target_triple: env!("VERGEN_CARGO_TARGET_TRIPLE"),
                        cargo_profile: profile_human(),
                    }
                }
            }

            impl std::fmt::Display for BuildInfo {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    writeln!(f, "Build Timestamp: {}", self.build_timestamp)?;
                    writeln!(f, "Build Version: {}", self.build_version)?;
                    if let Some(sha) = self.commit_sha {
                        write!(f, "Commit SHA: {}", sha)?;
                        match self.commit_dirty {
                            Some("true") => writeln!(f, "-dirty")?,
                            _ => writeln!(f)?,
                        }
                    }
                    if let Some(commit_date) = self.commit_date {
                        writeln!(f, "Commit Date: {}", commit_date)?;
                    }

                    if let Some(commit_branch) = self.commit_branch {
                        writeln!(f, "Commit Branch: {}", commit_branch)?;
                    }
                    writeln!(f, "rustc Version: {}", self.rustc_version)?;
                    writeln!(f, "rustc Channel: {}", self.rustc_channel)?;
                    writeln!(f, "rustc Host Triple: {}", self.rustc_host_triple)?;
                    writeln!(f, "cargo Target Triple: {}", self.cargo_target_triple)?;
                    write!(f, "cargo Profile: {}", self.cargo_profile)
                }
            }
        }
    };
}

/// Collects the information that can be used later to identify the version of this crate.
///
/// # Note
///
/// This should be used inside the build script of your crate.
pub fn collect_build_information() -> anyhow::Result<()> {
    let mut emitter = Emitter::default();
    let builder = emitter
        .add_instructions(&CargoBuilder::all_cargo().context("Failed to build cargo variables")?)
        .context("Failed to add cargo instructions")?
        .add_instructions(&BuildBuilder::all_build().context("Failed to build builder variables")?)
        .context("Failed to add builder variables")?
        .add_instructions(&RustcBuilder::all_rustc().context("Failed to build rustc variables")?)
        .context("Failed to add rustc variables")?;

    if is_contained_in_git()? {
        builder
            .add_instructions(&GixBuilder::all_git().context("Failed to build git variables")?)
            .context("Failed to add git variables")?;
    }
    builder.emit().context("Failed to emit")?;

    Ok(())
}

/// Checks whether the current or one of the parent directories contains a `.git` entry.
fn is_contained_in_git() -> anyhow::Result<bool> {
    let current_dir = std::env::current_dir().context("Failed to get current dir")?;
    let mut parents = vec![];
    let mut path = &*current_dir
        .canonicalize()
        .context("Failed to canonicalize path")?;
    parents.push(path);
    while let Some(parent) = path.parent() {
        parents.push(parent);
        path = parent;
    }
    for parent in parents.into_iter().rev() {
        if parent.join(".git").exists() {
            return Ok(true);
        }
    }

    println!("cargo::warning=No .git directory found");
    Ok(false)
}
