use std::cmp::Ordering;

use anyhow::Result;

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
#[derive(Clone, Debug, Default, clap::ValueEnum)]
pub enum SortType {
    /// Sort by name.
    #[default]
    Name,
    /// Sort by creation date.
    Created,
    /// Sort by last modified.
    Modified,
}

impl Sorter for SortType {
    fn sort(&self, a: &Entry, b: &Entry) -> Result<Ordering> {
        match self {
            Self::Name => Name.sort(a, b),
            Self::Created => Created.sort(a, b),
            Self::Modified => Modified.sort(a, b),
        }
    }
}

/// Sort by name.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Name;

impl Sorter for Name {
    fn sort(&self, a: &Entry, b: &Entry) -> Result<Ordering> {
        let a_path = a.path.as_os_str().to_ascii_lowercase();
        let b_path = b.path.as_os_str().to_ascii_lowercase();

        Ok(a_path.cmp(&b_path))
    }
}

/// Sort by creation date.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Created;

impl Sorter for Created {
    fn sort(&self, a: &Entry, b: &Entry) -> Result<Ordering> {
        let a_time = a.data.created()?;
        let b_time = b.data.created()?;

        Ok(a_time.cmp(&b_time))
    }
}

/// Sort by last modified.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Modified;

impl Sorter for Modified {
    fn sort(&self, a: &Entry, b: &Entry) -> Result<Ordering> {
        let a_time = a.data.modified()?;
        let b_time = b.data.modified()?;

        Ok(a_time.cmp(&b_time))
    }
}

/// Sort directories earlier.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct HoistDirs;

impl Sorter for HoistDirs {
    fn sort(&self, a: &Entry, b: &Entry) -> Result<Ordering> {
        match (a.data.is_dir(), b.data.is_dir()) {
            (true, false) => Ok(Ordering::Less),
            (false, true) => Ok(Ordering::Greater),
            _ => Ok(Ordering::Equal),
        }
    }
}
