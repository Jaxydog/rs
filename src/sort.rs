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

use core::cmp::Ordering;
use std::io::Result;

use crate::Entry;

/// A type that sorts entries.
pub trait Sorter {
    /// Sorts two entries.
    ///
    /// # Errors
    ///
    /// This function will return an error if sorting fails.
    fn sort(&self, a: &Entry, b: &Entry) -> Result<Ordering>;
}

/// Sorting types.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum SortType {
    /// Sort by name.
    #[default]
    Name,
    /// Sort by size.
    Size,
    /// Sort by creation date.
    Created,
    /// Sort by last modified.
    Modified,
}

impl Sorter for SortType {
    fn sort(&self, a: &Entry, b: &Entry) -> Result<Ordering> {
        match self {
            Self::Name => SortName.sort(a, b),
            Self::Size => SortSize.sort(a, b),
            Self::Created => SortCreated.sort(a, b),
            Self::Modified => SortModified.sort(a, b),
        }
    }
}

/// Sort by name.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SortName;

impl Sorter for SortName {
    fn sort(&self, a: &Entry, b: &Entry) -> Result<Ordering> {
        let a_path = a.path.as_os_str().to_ascii_lowercase();
        let b_path = b.path.as_os_str().to_ascii_lowercase();

        Ok(a_path.cmp(&b_path))
    }
}

/// Sort by size.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SortSize;

impl Sorter for SortSize {
    fn sort(&self, a: &Entry, b: &Entry) -> Result<Ordering> {
        Ok(a.data.len().cmp(&b.data.len()).reverse())
    }
}

/// Sort by creation date.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SortCreated;

impl Sorter for SortCreated {
    fn sort(&self, a: &Entry, b: &Entry) -> Result<Ordering> {
        let a_time = a.data.created()?;
        let b_time = b.data.created()?;

        Ok(a_time.cmp(&b_time).reverse())
    }
}

/// Sort by last modified.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SortModified;

impl Sorter for SortModified {
    fn sort(&self, a: &Entry, b: &Entry) -> Result<Ordering> {
        let a_time = a.data.modified()?;
        let b_time = b.data.modified()?;

        Ok(a_time.cmp(&b_time).reverse())
    }
}

/// Hoisting types.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum HoistType {
    /// Don't hoist anything.
    #[default]
    None,
    /// Hoist directories.
    Directories,
    /// Hoist hidden files.
    Hidden,
    /// Hoist symbolic links.
    Symlinks,
}

impl Sorter for HoistType {
    fn sort(&self, a: &Entry, b: &Entry) -> Result<Ordering> {
        match self {
            Self::None => Ok(Ordering::Equal),
            Self::Directories => HoistDirectories.sort(a, b),
            Self::Hidden => HoistHidden.sort(a, b),
            Self::Symlinks => HoistSymlinks.sort(a, b),
        }
    }
}

/// Sort directories earlier.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HoistDirectories;

impl Sorter for HoistDirectories {
    fn sort(&self, a: &Entry, b: &Entry) -> Result<Ordering> {
        match (a.data.is_dir(), b.data.is_dir()) {
            (true, false) => Ok(Ordering::Less),
            (false, true) => Ok(Ordering::Greater),
            _ => Ok(Ordering::Equal),
        }
    }
}

/// Sort directories earlier.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HoistHidden;

impl Sorter for HoistHidden {
    fn sort(&self, a: &Entry, b: &Entry) -> Result<Ordering> {
        let Some(a_name) = a.path.file_name() else { return Ok(Ordering::Equal) };
        let Some(b_name) = b.path.file_name() else { return Ok(Ordering::Equal) };

        match (a_name.to_string_lossy().starts_with('.'), b_name.to_string_lossy().starts_with('.')) {
            (true, false) => Ok(Ordering::Less),
            (false, true) => Ok(Ordering::Greater),
            _ => Ok(Ordering::Equal),
        }
    }
}

/// Sort symbolic links earlier.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HoistSymlinks;

impl Sorter for HoistSymlinks {
    fn sort(&self, a: &Entry, b: &Entry) -> Result<Ordering> {
        match (a.data.is_symlink(), b.data.is_symlink()) {
            (true, false) => Ok(Ordering::Less),
            (false, true) => Ok(Ordering::Greater),
            _ => Ok(Ordering::Equal),
        }
    }
}
