use core::fmt::Display;
use std::io::{Result, Write};
use std::path::MAIN_SEPARATOR;

use is_executable::IsExecutable;

use super::Displayer;
use crate::{cwrite, Entry};

/// Displays an entry's name.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Name {
    /// Whether to resolve symbolic links.
    pub resolve_symlinks: bool,
    /// Whether to trim file paths.
    pub trim_file_paths: bool,
}

#[allow(clippy::unused_self)]
impl Name {
    /// Creates a new [`Name`].
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

        cwrite!(bright_cyan; f, "{name}")?;

        if !self.resolve_symlinks {
            return Ok(());
        }

        let Ok(path) = std::fs::read_link(&entry.path) else {
            return fail(f, "N/A");
        };

        let resolve_path = entry.path.parent().map_or_else(|| path.clone(), |p| p.join(&path));

        if !std::fs::exists(&resolve_path).is_ok_and(core::convert::identity) {
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
        cwrite!(bright_blue; f, "{name}")?;

        if !name.ends_with(MAIN_SEPARATOR) {
            cwrite!(bright_blue; f, "{MAIN_SEPARATOR}")?;
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
            cwrite!(bright_green; f, "{name}")?;
            cwrite!(white; f, "*")?;
        } else {
            cwrite!(white; f, "{name}")?;
        }

        Ok(())
    }
}

impl Displayer for Name {
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
