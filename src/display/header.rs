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

use std::io::{Result, Write};

use crate::{arguments::Arguments, cwrite, Entry};

use super::{Displayer, HasColor};

/// Displays a directory header.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeaderDisplay<'ar> {
    /// The program's arguments.
    arguments: &'ar Arguments,
}

impl<'ar> HeaderDisplay<'ar> {
    /// Creates a new [`HeaderDisplay`].
    #[must_use]
    pub const fn new(arguments: &'ar Arguments) -> Self {
        Self { arguments }
    }
}

impl HasColor for HeaderDisplay<'_> {
    fn has_color(&self) -> Option<bool> {
        self.arguments.color
    }
}

impl Displayer for HeaderDisplay<'_> {
    fn show<W: Write>(&self, f: &mut W, entry: &Entry) -> Result<()> {
        cwrite!(self, bright_blue; f, "{}", entry.path.to_string_lossy())?;

        f.write_all(b":")
    }
}
