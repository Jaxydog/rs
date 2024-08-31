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

use std::fs::{DirEntry, Metadata, ReadDir};
use std::io::{Result, StderrLock, StdoutLock, Write};
use std::path::{Path, PathBuf};

use arguments::Arguments;
use display::{Displayer, HeaderDisplay, ModifiedDisplay, NameDisplay, PermissionsDisplay, SizeDisplay};
use sort::{HoistType, SortType, Sorter};

/// Defines the application's command-line arguments and handles parsing.
pub mod arguments;
/// Provides interfaces for displaying information.
pub mod display;
/// Provides interfaces for sorting entries.
pub mod sort;

/// A file system entry.
///
/// This is used to provide easy access to file metadata to [`Displayer`] implementations without making additional OS
/// calls if at all possible.
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

/// Returns an iterator over entries for the given path.
///
/// # Errors
///
/// This function will return an error if the iterator could not be created.
fn entries_iterator(
    stdout: &mut StdoutLock,
    stderr: &mut StderrLock,
    path: impl AsRef<Path>,
) -> Result<Option<ReadDir>> {
    let path = path.as_ref();

    if !path.try_exists()? {
        writeln!(stderr, "Invalid path '{}'.", path.to_string_lossy())?;

        return Ok(None);
    }
    if path.is_file() {
        writeln!(stdout, "'{}' is a file.", path.to_string_lossy())?;

        return Ok(None);
    }
    if path.is_symlink() {
        return self::entries_iterator(stdout, stderr, std::fs::read_link(path)?);
    }

    std::fs::read_dir(path).map(Some)
}

/// Returns a list of resolved entries to list.
///
/// # Panics
///
/// Panics if an error message could not be written to standard error during sorting.
///
/// # Errors
///
/// This function will return an error if the entries could not be resolved.
pub fn entries_list(
    arguments: &Arguments,
    stdout: &mut StdoutLock,
    stderr: &mut StderrLock,
    path: impl AsRef<Path>,
) -> Result<Option<Box<[Entry]>>> {
    let Some(iterator) = self::entries_iterator(stdout, stderr, path)? else {
        return Ok(None);
    };

    let mut entries = iterator.map(|v| v.and_then(Entry::try_from)).collect::<Result<Vec<_>>>()?;

    if !arguments.show_hidden {
        entries.retain(|entry| {
            let Some(name) = entry.path.file_name() else { return true };

            !name.to_string_lossy().starts_with('.')
        });
    }

    entries.sort_unstable_by(|a, b| {
        let hoisted = arguments.hoist_function.sort(a, b).unwrap_or_else(|error| {
            writeln!(stderr, "Failed to hoist entries: {error}").unwrap();

            core::cmp::Ordering::Equal
        });
        let sorted = arguments.sort_function.sort(a, b).unwrap_or_else(|error| {
            writeln!(stderr, "Failed to sort entries: {error}").unwrap();

            core::cmp::Ordering::Equal
        });

        hoisted.then(if arguments.sort_reversed { sorted.reverse() } else { sorted })
    });

    Ok(Some(entries.into_boxed_slice()))
}

/// Displays a list of entries.
///
/// # Errors
///
/// This function will return an error if the listing fails to display.
pub fn show(arguments: &Arguments, stdout: &mut StdoutLock, iterator: impl IntoIterator<Item = Entry>) -> Result<()> {
    let name = NameDisplay::new(arguments);
    let permissions = arguments.show_permissions.then(|| PermissionsDisplay::new(arguments));
    let size = arguments.show_sizes.then(|| SizeDisplay::new(arguments));
    let modified = arguments.show_modified.then(|| ModifiedDisplay::new(arguments));

    for ref entry in iterator {
        if let Some(ref displayer) = permissions {
            displayer.show(stdout, entry)?;

            stdout.write_all(b" ")?;
        };
        if let Some(ref displayer) = size {
            displayer.show(stdout, entry)?;

            stdout.write_all(b" ")?;
        };
        if let Some(ref displayer) = modified {
            displayer.show(stdout, entry)?;

            stdout.write_all(b" ")?;
        };

        name.show(stdout, entry)?;

        stdout.write_all(b"\n")?;
    }

    Ok(())
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

    if arguments.paths.is_empty() {
        let directory = std::env::current_dir()?;
        let Some(entries) = self::entries_list(&arguments, &mut stdout, &mut stderr, directory)? else {
            return Ok(());
        };

        stderr.flush()?;

        self::show(&arguments, &mut stdout, entries)?;

        return stdout.flush();
    }

    let header = HeaderDisplay::new(&arguments);

    for path in &arguments.paths {
        let Some(entries) = self::entries_list(&arguments, &mut stdout, &mut stderr, path)? else {
            continue;
        };

        stderr.flush()?;

        let entry = Entry::new(PathBuf::from(&(**path)), path.metadata()?);

        header.show(&mut stdout, &entry)?;

        stdout.write_all(b"\n")?;

        self::show(&arguments, &mut stdout, entries)?;

        stdout.write_all(b"\n")?;
    }

    stdout.flush()
}
