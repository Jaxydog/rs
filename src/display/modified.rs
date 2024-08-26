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

use time::format_description::FormatItem;
use time::{OffsetDateTime, UtcOffset};

use crate::cwrite;

use super::Displayer;

/// A human-friendly format.
const HUMAN_FORMAT: &[FormatItem] = time::macros::format_description!(
    version = 2,
    "[day padding:space] [month repr:short] '[year repr:last_two] [hour padding:space repr:24]:[minute padding:zero]"
);
/// A more machine-friendly format.
const MACHINE_FORMAT: &[FormatItem] = time::macros::format_description!(
    version = 2,
    "[year]-[month padding:zero]-[day padding:zero] [hour padding:zero repr:24]:[minute padding:zero]"
);

/// Display's an entry's modification date.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ModifiedDisplay {
    /// Whether to use human-readable units.
    pub human_readable: bool,
}

impl ModifiedDisplay {
    /// Creates a new [`ModifiedDisplay`].
    #[must_use]
    pub const fn new(human_readable: bool) -> Self {
        Self { human_readable }
    }
}

impl Displayer for ModifiedDisplay {
    fn show<W: Write>(&self, f: &mut W, entry: &crate::Entry) -> Result<()> {
        let time = entry.data.modified()?;
        let mut time = OffsetDateTime::from(time);

        if let Ok(offset) = UtcOffset::current_local_offset() {
            time = time.to_offset(offset);
        }

        let format = if self.human_readable { HUMAN_FORMAT } else { MACHINE_FORMAT };

        cwrite!(bright_blue; f, "{}", time.format(format).expect("the compiled format is incorrectly defined"))
    }
}
