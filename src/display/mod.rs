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

pub use header::HeaderDisplay;
pub use modified::ModifiedDisplay;
pub use name::NameDisplay;
pub use permissions::PermissionsDisplay;
pub use size::SizeDisplay;

use crate::Entry;

/// Defines the header display.
mod header;
/// Defines the modified display.
mod modified;
/// Defines the name display.
mod name;
/// Defines the permissions display.
mod permissions;
/// Defines the size display.
mod size;

/// A type that determines whether to display using color.
pub trait HasColor {
    /// Whether to display color.
    ///
    /// This usually will not need to be called directly; it should be preferred to call
    /// the [`cwrite!`](<crate::cwrite>) or [`cwriteln!`](<crate::cwriteln>) macros instead.
    fn has_color(&self) -> Option<bool>;
}

impl<T: HasColor> HasColor for &T {
    fn has_color(&self) -> Option<bool> {
        <T as HasColor>::has_color(self)
    }
}

/// A type that displays entries.
pub trait Displayer {
    /// Displays an entry.
    ///
    /// # Errors
    ///
    /// This function will return an error if the entry could not be displayed.
    fn show<W: Write>(&self, f: &mut W, entry: &Entry) -> std::io::Result<()>;
}

impl<T: Displayer> Displayer for &T {
    fn show<W: Write>(&self, f: &mut W, entry: &Entry) -> std::io::Result<()> {
        <T as Displayer>::show(self, f, entry)
    }
}

/// Writes a format string to standard output, optionally using color.
///
/// # Examples
///
/// ```
/// # use crate::display::NameDisplay;
/// # fn main() -> std::io::Result<()> {
/// #
/// let arguments = crate::arguments::parse();
/// let display = NameDisplay::new(&arguments);
///
/// cprint!(display, red; "some text!")?;
/// #
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! cprint {
    ($self:expr, $color:ident; $($body:tt)*) => {
        $crate::cwrite!($self, $color, ::owo_colors::Stream::Stdout; &mut ::std::io::stdout(), $($body)*)
    };
}

/// Writes a format string to standard error, optionally using color.
///
/// # Examples
///
/// ```
/// # use crate::display::NameDisplay;
/// # fn main() -> std::io::Result<()> {
/// #
/// let arguments = crate::arguments::parse();
/// let display = NameDisplay::new(&arguments);
///
/// ceprint!(display, red; "some text!")?;
/// #
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! ceprint {
    ($self:expr, $color:ident; $($body:tt)*) => {
        $crate::cwrite!($self, $color, ::owo_colors::Stream::Stderr; &mut ::std::io::stderr(), $($body)*)
    };
}

/// Writes a format string to standard output, optionally using color, and appends a newline.
///
/// # Examples
///
/// ```
/// # use crate::display::NameDisplay;
/// # fn main() -> std::io::Result<()> {
/// #
/// let arguments = crate::arguments::parse();
/// let display = NameDisplay::new(&arguments);
///
/// cprintln!(display, red; "some text!")?;
/// #
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! cprintln {
    ($self:expr, $color:ident; $($body:tt)*) => {
        $crate::cwriteln!($self, $color, ::owo_colors::Stream::Stdout; &mut ::std::io::stdout(), $($body)*)
    };
}

/// Writes a format string to standard error, optionally using color, and appends a newline.
///
/// # Examples
///
/// ```
/// # use crate::display::NameDisplay;
/// # fn main() -> std::io::Result<()> {
/// #
/// let arguments = crate::arguments::parse();
/// let display = NameDisplay::new(&arguments);
///
/// ceprintln!(display, red; "some text!")?;
/// #
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! ceprintln {
    ($self:expr, $color:ident; $($body:tt)*) => {
        $crate::cwriteln!($self, $color, ::owo_colors::Stream::Stderr; &mut ::std::io::stderr(), $($body)*)
    };
}

/// Writes a format string to the given buffer, optionally using color.
///
/// # Examples
///
/// ```
/// # use crate::display::NameDisplay;
/// # fn main() -> std::io::Result<()> {
/// #
/// let arguments = crate::arguments::parse();
/// let mut stdout = std::io::stdout();
/// let display = NameDisplay::new(&arguments);
///
/// cwrite!(display, red; &mut stdout, "some text!")?;
/// #
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! cwrite {
    ($self:expr, $color:ident; $write:expr, $($body:tt)*) => {
        $crate::cwrite!($self, $color, ::owo_colors::Stream::Stdout; $write, $($body)*)
    };
    ($self:expr, $color:ident, $stream:expr; $write:expr, $($body:tt)*) => {
        match <_ as $crate::display::HasColor>::has_color(&$self) {
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
                    $stream,
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
/// # use crate::display::NameDisplay;
/// # fn main() -> std::io::Result<()> {
/// #
/// let arguments = crate::arguments::parse();
/// let mut stdout = std::io::stdout();
/// let display = NameDisplay::new(&arguments);
///
/// cwriteln!(display, red; &mut stdout, "some text!")?;
/// #
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! cwriteln {
    ($self:expr, $color:ident; $write:expr, $($body:tt)*) => {
        $crate::cwriteln!($self, $color, ::owo_colors::Stream::Stdout; $write, $($body)*)
    };
    ($self:expr, $color:ident, $stream:expr; $write:expr, $($body:tt)*) => {
        match <_ as $crate::display::HasColor>::has_color(&$self) {
            ::core::option::Option::Some(false) => ::core::writeln!($write, $($body)*),
            ::core::option::Option::Some(true) => ::core::writeln!(
                $write,
                "{}",
                <_ as ::owo_colors::OwoColorize>::$color(&::core::format_args!($($body)*))
            ),
            ::core::option::Option::None => ::core::writeln!(
                $write,
                "{}",
                <_ as ::owo_colors::OwoColorize>::if_supports_color(
                    &::core::format_args!($($body)*),
                    $stream,
                    |v| <_ as ::owo_colors::OwoColorize>::$color(v)
                )
            ),
        }
    };
}
