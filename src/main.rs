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
use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

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
    /// Display entries starting with '.'.
    #[arg(short = 'a', long = "all")]
    pub show_all: bool,
}

/// The program's entrypoint.
///
/// # Errors
///
/// This function will return an error if the program's execution fails in an un-recoverable manner.
pub fn main() -> Result<()> {
    let arguments = Arguments::parse();

    Ok(())
}
