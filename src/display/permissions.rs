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

use std::fs::Metadata;
use std::io::{Result, Write};

use super::{Displayer, HasColor};
use crate::arguments::Arguments;
use crate::{cwrite, Entry};

/// Displays an entry's permissions.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PermissionsDisplay<'ar> {
    /// The program's arguments.
    arguments: &'ar Arguments,
}

impl<'ar> PermissionsDisplay<'ar> {
    /// Creates a new [`PermissionsDisplay`].
    #[must_use]
    pub const fn new(arguments: &'ar Arguments) -> Self {
        Self { arguments }
    }

    /// Displays an entry's Unix permissions.
    ///
    /// # Errors
    ///
    /// This function will return an error if the permissions could not be displayed.
    #[cfg(target_family = "unix")]
    fn show_entry<W: Write>(&self, f: &mut W, entry: &Entry) -> Result<()> {
        const FILE_TYPE_MASK: u32 = 0o0_170_000;
        const SOCKET: u32 = 0o0_140_000;
        const SYMBOLIC_LINK: u32 = 0o0_120_000;
        const FILE: u32 = 0o0_100_000;
        const BLOCK_DEVICE: u32 = 0o0_060_000;
        const DIRECTORY: u32 = 0o0_040_000;
        const CHARACTER_DEVICE: u32 = 0o0_020_000;
        const FIFO_PIPE: u32 = 0o0_010_000;

        let mode = <Metadata as std::os::unix::fs::MetadataExt>::mode(&entry.data);
        let string = ::umask::Mode::from(mode).to_string();

        let character = match mode & FILE_TYPE_MASK {
            SOCKET => 's',
            SYMBOLIC_LINK => 'l',
            FILE => '-',
            BLOCK_DEVICE => 'b',
            DIRECTORY => 'd',
            CHARACTER_DEVICE => 'c',
            FIFO_PIPE => 'p',
            _ => '?',
        };

        self.show_char(f, character)?;

        for character in string.chars() {
            self.show_char(f, character)?;
        }

        Ok(())
    }

    /// Displays an entry's Windows permissions.
    ///
    /// # Errors
    ///
    /// This function will return an error if the permissions could not be displayed.
    #[cfg(target_family = "windows")]
    fn show_entry<W: Write>(&self, f: &mut W, entry: &Entry) -> Result<()> {
        let bits = <Metadata as std::os::windows::fs::MetadataExt>::file_attributes(&entry.data);
        let string = WindowsPermissions { bits }.to_string();

        if entry.data.is_symlink() {
            self.show_char(f, 'l')?;
        } else if entry.data.is_dir() {
            self.show_char(f, 'd')?;
        } else {
            self.show_char(f, '-')?;
        }

        for character in string.chars() {
            self.show_char(f, character)?;
        }

        Ok(())
    }

    /// Displays an entry's permission character.
    ///
    /// # Errors
    ///
    /// This function will return an error if the permission could not be displayed.
    #[cfg(target_family = "unix")]
    fn show_char<W: Write>(&self, f: &mut W, character: char) -> Result<()> {
        match character {
            // Read permission.
            c @ 'r' => cwrite!(self, bright_yellow; f, "{c}"),
            // Write permission.
            c @ 'w' => cwrite!(self, bright_red; f, "{c}"),
            // Execute permission.
            c @ 'x' => cwrite!(self, bright_green; f, "{c}"),
            // File / No value.
            c @ '-' => cwrite!(self, bright_black; f, "{c}"),
            // Directory.
            c @ 'd' => cwrite!(self, bright_blue; f, "{c}"),
            // Symbolic link.
            c @ 'l' => cwrite!(self, bright_cyan; f, "{c}"),
            // Socket.
            c @ 's' => cwrite!(self, bright_green; f, "{c}"),
            // Block device.
            c @ 'b' => cwrite!(self, bright_red; f, "{c}"),
            // Character device.
            c @ 'c' => cwrite!(self, bright_yellow; f, "{c}"),
            // FIFO pipe.
            c @ 'p' => cwrite!(self, bright_purple; f, "{c}"),
            // Anything else.
            unknown => cwrite!(self, bright_magenta; f, "{unknown}"),
        }
    }

    /// Displays an entry's permission character.
    ///
    /// # Errors
    ///
    /// This function will return an error if the permission could not be displayed.
    #[cfg(target_family = "windows")]
    fn show_char<W: Write>(&self, f: &mut W, character: char) -> Result<()> {
        match character {
            // Read-only.
            c @ 'r' => cwrite!(self, bright_yellow; f, "{c}"),
            // Archive.
            c @ 'a' => cwrite!(self, bright_red; f, "{c}"),
            // Hidden.
            c @ 'h' => cwrite!(self, bright_purple; f, "{c}"),
            // System.
            c @ 's' => cwrite!(self, bright_green; f, "{c}"),
            // File / No value.
            c @ '-' => cwrite!(self, bright_black; f, "{c}"),
            // Directory.
            c @ 'd' => cwrite!(self, bright_blue; f, "{c}"),
            // Symbolic link.
            c @ 'l' => cwrite!(self, bright_cyan; f, "{c}"),
            // Anything else.
            unknown => cwrite!(self, bright_magenta; f, "{unknown}"),
        }
    }
}

impl HasColor for PermissionsDisplay<'_> {
    fn has_color(&self) -> Option<bool> {
        self.arguments.color
    }
}

impl Displayer for PermissionsDisplay<'_> {
    fn show<W: Write>(&self, f: &mut W, entry: &Entry) -> Result<()> {
        cwrite!(self, bright_black; f, "[")?;

        self.show_entry(f, entry)?;

        cwrite!(self, bright_black; f, "]").map_err(Into::into)
    }
}

/// Parses out Windows permissions.
#[cfg(target_family = "windows")]
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct WindowsPermissions {
    /// The permission bits.
    pub bits: u32,
}

#[cfg(target_family = "windows")]
impl WindowsPermissions {
    /// Returns whether the given flag is set in the inner permission bits.
    #[inline]
    const fn has_flag(self, flag: u32) -> bool {
        self.bits & flag != 0
    }

    /// Returns whether this [`WindowsPermissions`] is read-only.
    #[must_use]
    pub const fn is_readonly(self) -> bool {
        self.has_flag(1 << 0)
    }

    /// Returns whether this [`WindowsPermissions`] is hidden.
    #[must_use]
    pub const fn is_hidden(self) -> bool {
        self.has_flag(1 << 1)
    }

    /// Returns whether this [`WindowsPermissions`] is a system entry.
    #[must_use]
    pub const fn is_system(self) -> bool {
        self.has_flag(1 << 2)
    }

    /// Returns whether this [`WindowsPermissions`] is an archive.
    #[must_use]
    pub const fn is_archive(self) -> bool {
        self.has_flag(1 << 4)
    }
}

#[cfg(target_family = "windows")]
impl core::fmt::Display for WindowsPermissions {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        #[inline]
        fn write(f: &mut core::fmt::Formatter<'_>, b: bool, c: char) -> core::fmt::Result {
            write!(f, "{}", if b { c } else { '-' })
        }

        write(f, self.is_readonly(), 'r')?;
        write(f, self.is_archive(), 'a')?;
        write(f, self.is_hidden(), 'h')?;
        write(f, self.is_system(), 's')?;

        Ok(())
    }
}
