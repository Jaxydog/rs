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

use core::fmt::Display;
use std::io::{Result, Write};

use super::Displayer;
use crate::{cwrite, Entry};

/// Displays an entry's name.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Size {
    /// Whether to use human-readable units.
    pub human_readable: bool,
}

impl Size {
    /// All accepted human-readable byte suffixes.
    pub const SUFFIXES: [&'static str; 7] = ["B  ", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB"];

    /// Creates a new [`Size`].
    #[must_use]
    pub const fn new(human_readable: bool) -> Self {
        Self { human_readable }
    }

    /// Displays the given value, aligned to the right and capped at 9 characters.
    ///
    /// # Errors
    ///
    /// This function will return an error if the value cannot be displayed.
    fn show_aligned<W, T>(&self, f: &mut W, v: T, dim: bool) -> Result<()>
    where
        W: Write,
        T: Display,
    {
        let output = if self.human_readable {
            format!("{v:>9}")
        } else {
            let string = v.to_string();

            if string.len() <= 9 {
                string
            } else {
                format!("{:>6}...", &string[..6])
            }
        };

        if dim {
            cwrite!(bright_black; f, "{output:>9}")
        } else {
            cwrite!(bright_green; f, "{output:>9}")
        }
    }

    /// Displays the given size in bytes in a human-readable format.
    ///
    /// # Errors
    ///
    /// This function will return an error if the value cannot be displayed.
    #[allow(clippy::cast_precision_loss)]
    fn show_human_readable<W: Write>(&self, f: &mut W, bytes: u64) -> Result<()> {
        if bytes == 0 {
            return self.show_aligned(f, format_args!("0 {}", Self::SUFFIXES[0]), false);
        }

        for (index, suffix) in Self::SUFFIXES.iter().enumerate() {
            let min_bound = 1 << (10 * index);
            let max_bound = 1 << (10 * (index + 1));
            let suffix_bounds = min_bound..max_bound;

            if suffix_bounds.contains(&bytes) {
                return if index == 0 {
                    self.show_aligned(f, format_args!("{bytes} {suffix}"), false)
                } else {
                    let value = bytes as f64 / min_bound as f64;

                    self.show_aligned(f, format_args!("{value:.1} {suffix}"), false)
                };
            }
        }

        self.show_aligned(f, bytes, false)
    }
}

impl Displayer for Size {
    fn show<W: Write>(&self, f: &mut W, entry: &Entry) -> Result<()> {
        if entry.data.is_dir() {
            return self.show_aligned(f, if self.human_readable { "- -  " } else { "-" }, true);
        }

        let bytes = entry.data.len();

        if self.human_readable {
            self.show_human_readable(f, bytes)
        } else {
            self.show_aligned(f, bytes, false)
        }
    }
}
