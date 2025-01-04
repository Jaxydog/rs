// SPDX-License-Identifier: AGPL-3.0-or-later
//
// Copyright © 2024—2025 Jaxydog
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

use std::io::{Result, Write};

use crate::{arguments::Arguments, cwrite, Entry};

use super::{Displayer, HasColor};

/// Displays an entry's file owner.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OwnerDisplay<'ar> {
    /// The program's arguments.
    arguments: &'ar Arguments,
}

impl<'ar> OwnerDisplay<'ar> {
    /// Creates a new [`OwnerDisplay`].
    #[must_use]
    pub const fn new(arguments: &'ar Arguments) -> Self {
        Self { arguments }
    }

    /// Returns the name of the given entry's owner.
    ///
    /// # Errors
    ///
    /// This function will return an error if the name could not be resolved.
    #[cfg(target_family = "unix")]
    fn get_owner_name(entry: &Entry) -> Result<Box<str>> {
        use std::os::unix::fs::MetadataExt;

        use nix::unistd::{Uid, User};

        let uid = entry.data.uid();
        let user = User::from_uid(Uid::from_raw(uid))?;

        Ok(user.map_or_else(|| "unknown".into(), |v| v.name.into_boxed_str()))
    }

    /// Returns the name of the given entry's owner.
    ///
    /// # Errors
    ///
    /// This function will return an error if the name could not be resolved.
    #[cfg(target_family = "windows")]
    fn get_owner_name(entry: &Entry) -> Result<Box<str>> {
        use windows_permissions::{
            constants::{SeObjectType, SecurityInformation},
            wrappers::{GetSecurityInfo, LookupAccountSid},
        };

        if entry.data.is_dir() {
            return Ok("-".into());
        }

        let file = std::fs::File::open(&entry.path)?;
        let descriptor = GetSecurityInfo(&file, SeObjectType::SE_FILE_OBJECT, SecurityInformation::Owner)?;
        let (name, _) = LookupAccountSid(descriptor.owner().expect("missing required data"))?;

        Ok(name.to_string_lossy().into())
    }
}

impl HasColor for OwnerDisplay<'_> {
    fn has_color(&self) -> Option<bool> {
        self.arguments.color
    }
}

impl Displayer for OwnerDisplay<'_> {
    fn show<W: Write>(&self, f: &mut W, entry: &Entry) -> Result<()> {
        cwrite!(self, bright_green; f, "{:>8}", Self::get_owner_name(entry)?)
    }
}
