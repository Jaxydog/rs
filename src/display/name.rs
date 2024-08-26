// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Copyright © 2024 Jaxydog
//
// This file is part of rs.
//
// rs is free software: you can redistribute it and/or modify it under the terms of the GNU Affero
// General Public License as published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// rs is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the
// implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero
// General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License along with rs. If not,
// see <https://www.gnu.org/licenses/>.

extern crate alloc;

use core::fmt::Display;
use std::io::{Result, Write};
use std::path::MAIN_SEPARATOR;

use is_executable::IsExecutable;

use super::Displayer;
use crate::{cwrite, Entry};

/// Displays an entry's name.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct NameDisplay {
    /// Whether to resolve symbolic links.
    pub resolve_symlinks: bool,
    /// Whether to trim file paths.
    pub trim_file_paths: bool,
}

#[allow(clippy::unused_self)]
impl NameDisplay {
    /// Creates a new [`NameDisplay`].
    #[must_use]
    pub const fn new(resolve_symlinks: bool) -> Self {
        Self { resolve_symlinks, trim_file_paths: true }
    }

    /// Displays a symlink file name within the given writer.
    ///
    /// # Errors
    ///
    /// This function will return an error if the entry fails to display.
    fn show_symlink<W: Write>(&self, f: &mut W, entry: &Entry, name: &str) -> Result<()> {
        #[inline]
        fn fail<W: Write, D: Display>(f: &mut W, v: D) -> Result<()> {
            cwrite!(bright_black; f, " ~> ")?;
            cwrite!(bright_red; f, "{v}")
        }

        if name.starts_with('.') {
            cwrite!(cyan; f, "{name}")?;
        } else {
            cwrite!(bright_cyan; f, "{name}")?;
        }

        if !self.resolve_symlinks {
            return Ok(());
        }

        let Ok(path) = std::fs::read_link(&entry.path) else {
            return fail(f, "N/A");
        };

        let resolve_path = entry.path.parent().map_or_else(|| path.clone(), |p| p.join(&path));

        if !resolve_path.try_exists().is_ok_and(core::convert::identity) {
            return fail(f, path.to_string_lossy());
        }

        let Ok(data) = std::fs::metadata(&resolve_path) else {
            return fail(f, path.to_string_lossy());
        };

        cwrite!(bright_black; f, " -> ")?;

        let mut copy = self.clone();

        copy.trim_file_paths = false;
        copy.show(f, &Entry { path: resolve_path, data })?;

        Ok(())
    }

    /// Displays a directory name within the given writer.
    ///
    /// # Errors
    ///
    /// This function will return an error if the entry fails to display.
    fn show_dir<W: Write>(&self, f: &mut W, name: &str) -> Result<()> {
        if name.starts_with('.') {
            cwrite!(blue; f, "{name}")?;

            if !name.ends_with(MAIN_SEPARATOR) {
                cwrite!(blue; f, "{MAIN_SEPARATOR}")?;
            }
        } else {
            cwrite!(bright_blue; f, "{name}")?;

            if !name.ends_with(MAIN_SEPARATOR) {
                cwrite!(bright_blue; f, "{MAIN_SEPARATOR}")?;
            }
        }

        Ok(())
    }

    /// Displays a directory name within the given writer.
    ///
    /// # Errors
    ///
    /// This function will return an error if the entry fails to display.
    fn show_file<W: Write>(&self, f: &mut W, entry: &Entry, name: &str) -> Result<()> {
        if entry.path.is_executable() {
            if entry.path.file_stem().is_some_and(|p| p.to_string_lossy().starts_with('.')) {
                cwrite!(green; f, "{name}")?;
            } else {
                cwrite!(bright_green; f, "{name}")?;
            }

            cwrite!(white; f, "*")
        } else if entry.path.file_stem().is_some_and(|p| p.to_string_lossy().starts_with('.')) {
            cwrite!(bright_black; f, "{name}")
        } else {
            cwrite!(white; f, "{name}")
        }
    }
}

impl Displayer for NameDisplay {
    fn show<W: Write>(&self, f: &mut W, entry: &Entry) -> Result<()> {
        let name = if self.trim_file_paths {
            let os_name = entry.path.file_name().unwrap_or(entry.path.as_os_str());

            os_name.to_string_lossy().into_owned()
        } else {
            entry.path.to_string_lossy().into_owned()
        };

        if entry.data.is_symlink() {
            self.show_symlink(f, entry, &name)
        } else if entry.data.is_dir() {
            self.show_dir(f, &name)
        } else {
            self.show_file(f, entry, &name)
        }
    }
}
