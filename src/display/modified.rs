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

use time::{format_description::FormatItem, OffsetDateTime, UtcOffset};

use crate::cwrite;

use super::Displayer;

const FORMAT: &[FormatItem] = time::macros::format_description!(
    version = 2,
    "[day padding:space] [month repr:short] '[year repr:last_two] [hour padding:space repr:24]:[minute padding:zero]"
);

/// Display's an entry's modification date.
pub struct Modified {}

impl Displayer for Modified {
    fn show<W: Write>(&self, f: &mut W, entry: &crate::Entry) -> Result<()> {
        let time = entry.data.modified()?;
        let mut time = OffsetDateTime::from(time);

        if let Ok(offset) = UtcOffset::current_local_offset() {
            time = time.to_offset(offset);
        }

        cwrite!(bright_blue; f, "{}", time.format(FORMAT).expect("the compiled format is incorrectly defined"))
    }
}
