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

use std::io::Write;

pub use self::modified::ModifiedDisplay;
pub use self::name::NameDisplay;
pub use self::permissions::PermissionsDisplay;
pub use self::size::SizeDisplay;
use crate::Entry;

/// Defines the modified display.
mod modified;
/// Defines the name display.
mod name;
/// Defines the permissions display.
mod permissions;
/// Defines the size display.
mod size;

/// A type that displays entries.
pub trait Displayer {
    /// Whether to display color.
    ///
    /// This usually will not need to be called directly; it should be preferred to call the [`cwrite!`]
    /// or [`cwriteln!`] macros instead.
    fn color(&self) -> Option<bool>;

    /// Displays an entry.
    ///
    /// # Errors
    ///
    /// This function will return an error if the entry could not be displayed.
    fn show<W: Write>(&self, f: &mut W, entry: &Entry) -> std::io::Result<()>;
}

impl<T: Displayer> Displayer for &T {
    fn color(&self) -> Option<bool> {
        <T as Displayer>::color(self)
    }

    fn show<W: Write>(&self, f: &mut W, entry: &Entry) -> std::io::Result<()> {
        <T as Displayer>::show(self, f, entry)
    }
}

/// Writes a format string to the given buffer, optionally using color.
///
/// # Examples
///
/// ```
/// use crate::display::NameDisplay;
///
/// let arguments = crate::arguments::parse();
/// let mut stdout = std::io::stdout();
/// let display = NameDisplay::new(&arguments);
///
/// cwrite!(display, red; &mut stdout, "some text!").expect("writing should not fail");
/// ```
#[macro_export]
macro_rules! cwrite {
    ($self:expr, $color:ident; $write:expr, $($body:tt)*) => {
        match <_ as $crate::display::Displayer>::color(&$self) {
            ::core::option::Option::Some(false) => ::core::write!($write, $($body)*),
            ::core::option::Option::Some(true) => ::core::write!(
                $write,
                "{}",
                <_ as ::owo_colors::OwoColorize>::$color(&::core::format_args!($($body)*))
            ),
            ::core::option::Option::None => ::core::write!(
                $write,
                "{}",
                <_ as ::owo_colors::OwoColorize>::if_supports_color(
                    &::core::format_args!($($body)*),
                    ::owo_colors::Stream::Stdout,
                    |v| <_ as ::owo_colors::OwoColorize>::$color(v)
                )
            ),
        }
    };
}

/// Writes a format string to the given buffer, optionally using color, and appends a newline.
///
/// # Examples
///
/// ```
/// use crate::display::NameDisplay;
///
/// let arguments = crate::arguments::parse();
/// let mut stdout = std::io::stdout();
/// let display = NameDisplay::new(&arguments);
///
/// cwriteln!(display, red; &mut stdout, "some text!").expect("writing should not fail");
/// ```
#[macro_export]
macro_rules! cwriteln {
    ($self:expr, $color:ident; $write:expr, $($body:tt)*) => {
        match <_ as $crate::display::Displayer>::color(&$self) {
            ::core::option::Option::Some(false) => ::core::writeln!($write, $($body)*),
            ::core::option::Option::Some(true) => ::core::writeln!(
                $write,
                "{}",
                <_ as ::owo_colors::OwoColorize>::$color(::core::format_args!($($body)*))
            ),
            ::core::option::Option::None => ::core::writeln!(
                $write,
                "{}",
                <_ as ::owo_colors::OwoColorize>::if_supports_color(
                    ::core::format_args!($($body)*),
                    ::owo_colors::Stream::Stdout,
                    |v| <_ as ::owo_colors::OwoColorize>::$color(v)
                )
            ),
        }
    };
}
