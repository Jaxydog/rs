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

//! A Rust implementation of 'ls'.
#![deny(clippy::unwrap_used, unsafe_code)]
#![warn(clippy::cargo, clippy::nursery, clippy::pedantic, clippy::todo, missing_docs)]
#![warn(clippy::alloc_instead_of_core, clippy::std_instead_of_alloc, clippy::std_instead_of_core)]
#![allow(clippy::module_name_repetitions)]

extern crate alloc;

use std::fs::{DirEntry, Metadata};
use std::io::{Result, Write};
use std::path::PathBuf;

use display::{Displayer, ModifiedDisplay, NameDisplay, PermissionsDisplay, SizeDisplay};
use sort::{HoistType, SortType, Sorter};

/// Defines the application's command-line arguments and handles parsing.
pub mod arguments;
/// Provides interfaces for displaying information.
pub mod display;
/// Provides interfaces for sorting entries.
pub mod sort;

/// A file system entry.
#[derive(Clone, Debug)]
pub struct Entry {
    /// The entry's path.
    pub path: PathBuf,
    /// The entry's data.
    pub data: Metadata,
}

impl Entry {
    /// Creates a new [`Entry`].
    #[must_use]
    pub const fn new(path: PathBuf, data: Metadata) -> Self {
        Self { path, data }
    }
}

impl TryFrom<DirEntry> for Entry {
    type Error = std::io::Error;

    fn try_from(value: DirEntry) -> Result<Self> {
        Ok(Self::new(value.path(), value.metadata()?))
    }
}

/// The program's entrypoint.
///
/// # Errors
///
/// This function will return an error if the program's execution fails in an un-recoverable manner.
pub fn main() -> Result<()> {
    let mut arguments = self::arguments::parse();

    if arguments.sort_function == SortType::Size && arguments.hoist_function == HoistType::None {
        arguments.hoist_function = HoistType::Directories;
    }

    let mut stdout = std::io::stdout().lock();
    let mut stderr = std::io::stderr().lock();
    let mut path = arguments.path.clone().unwrap_or_else(|| PathBuf::from(".").into_boxed_path());

    if !path.try_exists()? {
        return writeln!(&mut stderr, "Invalid path '{}'.", path.to_string_lossy());
    }
    if path.is_symlink() {
        path = std::fs::read_link(path)?.into_boxed_path();
    }
    if path.is_file() {
        return writeln!(&mut stdout, "'{}' is a file.", path.to_string_lossy());
    }

    let mut entries = std::fs::read_dir(&path)?.map(|v| v.and_then(Entry::try_from)).collect::<Result<Vec<_>>>()?;

    if !arguments.show_hidden {
        entries.retain(|entry| {
            let Some(name) = entry.path.file_name() else { return true };

            !name.to_string_lossy().starts_with('.')
        });
    }

    entries.sort_unstable_by(|a, b| {
        let hoisted = arguments.hoist_function.sort(a, b).unwrap_or_else(|error| {
            writeln!(&mut stderr, "Failed to hoist entries: {error}").unwrap();

            core::cmp::Ordering::Equal
        });
        let sorted = arguments.sort_function.sort(a, b).unwrap_or_else(|error| {
            writeln!(&mut stderr, "Failed to sort entries: {error}").unwrap();

            core::cmp::Ordering::Equal
        });

        hoisted.then(if arguments.sort_reversed { sorted.reverse() } else { sorted })
    });

    stderr.flush()?;
    drop(stderr);

    let name = NameDisplay::new(&arguments);
    let permissions = arguments.show_permissions.then(|| PermissionsDisplay::new(&arguments));
    let size = arguments.show_sizes.then(|| SizeDisplay::new(&arguments));
    let modified = arguments.show_modified.then(|| ModifiedDisplay::new(&arguments));

    for ref entry in entries {
        if let Some(ref displayer) = permissions {
            displayer.show(&mut stdout, entry)?;
            stdout.write_all(b" ")?;
        };
        if let Some(ref displayer) = size {
            displayer.show(&mut stdout, entry)?;
            stdout.write_all(b" ")?;
        };
        if let Some(ref displayer) = modified {
            displayer.show(&mut stdout, entry)?;
            stdout.write_all(b" ")?;
        };

        name.show(&mut stdout, entry)?;
        stdout.write_all(b"\n")?;
    }

    stdout.flush()
}
