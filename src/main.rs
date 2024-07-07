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
#![allow(clippy::module_name_repetitions)]

use std::fs::{DirEntry, Metadata};
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::Parser;
use display::Displayer;
use sort::{HoistType, SortType, Sorter};

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

    fn try_from(value: DirEntry) -> Result<Self, Self::Error> {
        Ok(Self::new(value.path(), value.metadata()?))
    }
}

/// The program's command-line arguments.
#[non_exhaustive]
#[derive(Clone, Debug, clap::Parser)]
#[command(about, version, long_about = None)]
pub struct Arguments {
    /// The path to list.
    #[arg(default_value = ".")]
    pub path: Box<Path>,
    /// Display entries starting with '.'.
    #[arg(short = 'a', long = "all")]
    pub all: bool,
    /// Sorts entries using the given method.
    #[arg(short = 's', long = "sort-by", default_value = "name")]
    pub sort_by: SortType,
    /// Groups entries at the top of the listing by the given type.
    #[arg(short = 'H', long = "hoist", default_value = "none")]
    pub hoist_by: HoistType,
}

/// The program's entrypoint.
///
/// # Errors
///
/// This function will return an error if the program's execution fails in an un-recoverable manner.
pub fn main() -> Result<()> {
    let mut arguments = Arguments::parse();

    if arguments.sort_by == SortType::Size && arguments.hoist_by == HoistType::None {
        arguments.hoist_by = HoistType::Directories;
    }

    println!("{arguments:?}");

    let mut stdout = std::io::stdout().lock();
    let mut entries = std::fs::read_dir(&arguments.path)?
        .map(|v| v.and_then(Entry::try_from))
        .try_fold(Vec::new(), |mut vec, result| {
            vec.push(result?);

            Ok::<_, std::io::Error>(vec)
        })?;

    entries.sort_unstable_by(|a, b| {
        let hoisted = arguments.hoist_by.sort(a, b).unwrap_or_else(|error| {
            eprintln!("failed to hoist - {error}");

            std::cmp::Ordering::Equal
        });
        let sorted = arguments.sort_by.sort(a, b).unwrap_or_else(|error| {
            eprintln!("failed to sort - {error}");

            std::cmp::Ordering::Equal
        });

        hoisted.then(sorted)
    });

    let name = display::Name::new(true);
    let size = display::Size::new(true);

    for entry in &entries {
        size.show(&mut stdout, entry)?;
        stdout.write_all(&[b' '])?;
        name.show(&mut stdout, entry)?;
        stdout.write_all(&[b'\n'])?;
    }

    stdout.flush().map_err(Into::into)
}
