extern crate alloc;

use std::io::Write;

pub use self::name::Name;
pub use self::permissions::Permissions;
pub use self::size::Size;
use crate::Entry;

/// Defines the name display.
mod name;
/// Defines the permissions display.
mod permissions;
/// Defines the size display.
mod size;

/// A type that displays entries.
pub trait Displayer {
    /// Displays an entry.
    ///
    /// # Errors
    ///
    /// This function will return an error if the entry could not be displayed.
    fn show<W: Write>(&self, f: &mut W, entry: &Entry) -> std::io::Result<()>;
}

/// Applies a color to the given displayable value.
///
/// # Examples
///
/// ```
/// let string = color!(red; "This is red now!");
/// ```
#[macro_export]
macro_rules! color {
    ($color:ident; $display:expr) => {
        <_ as ::owo_colors::OwoColorize>::if_supports_color(&$display, ::owo_colors::Stream::Stdout, |v| {
            <_ as ::owo_colors::OwoColorize>::$color(v)
        })
    };
}

/// Writes to the given implementer of [`Write`] in the given color.
///
/// # Examples
///
/// ```
/// cwrite!(red; f, "This is red now!")?;
/// ```
#[macro_export]
macro_rules! cwrite {
    ($color:ident; $writer:expr, $($args:tt)+) => {
        ::core::write!($writer, "{}", $crate::color!($color; ::core::format_args!($($args)+)))
    };
}

/// Writes to the given implementer of [`Write`] in the given color and appends a newline.
///
/// # Examples
///
/// ```
/// cwriteln!(red; f, "This is red now!")?;
/// ```
#[macro_export]
macro_rules! cwriteln {
    ($color:ident; $writer:expr, $($args:tt)+) => {
        ::core::writeln!($writer, "{}", $crate::color!($color; ::core::format_args!($($args)+)))
    };
}
