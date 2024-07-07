use std::convert::identity;
use std::fmt::Display;
use std::io::Write;
use std::path::MAIN_SEPARATOR;

use anyhow::Result;
use is_executable::IsExecutable;

use crate::{cwrite, Entry};

/// A type that displays entries.
pub trait Displayer {
    /// Displays an entry.
    ///
    /// # Errors
    ///
    /// This function will return an error if the entry could not be displayed.
    fn show<W: Write>(&self, f: &mut W, entry: &Entry) -> Result<()>;
}

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
            cwrite!(bright_red; f, "{v}").map_err(Into::into)
        }

        cwrite!(bright_cyan; f, "{name}")?;

        let Ok(path) = std::fs::read_link(&entry.path) else {
            return fail(f, "N/A");
        };

        if !std::fs::exists(&path).is_ok_and(identity) {
            return fail(f, path.to_string_lossy());
        }

        let Ok(data) = std::fs::metadata(&path) else {
            return fail(f, path.to_string_lossy());
        };

        cwrite!(bright_black; f, " -> ")?;

        let mut copy = self.clone();

        copy.trim_file_paths = false;
        copy.show(f, &Entry { path, data })?;

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

/// Displays an entry's name.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Size {
    /// Whether to use human-readable units.
    pub human_readable: bool,
}

impl Size {
    /// All accepted human-readable byte suffixes.
    pub const SUFFIXES: [&str; 7] = ["B  ", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB"];

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
    fn show_aligned<W: Write, T: Display>(&self, f: &mut W, v: T, dim: bool) -> Result<()> {
        let output = if self.human_readable {
            format!("{v:>9}")
        } else {
            let string = v.to_string();

            if string.len() <= 9 { string } else { format!("{:>6}...", &string[.. 6]) }
        };

        if dim {
            cwrite!(bright_black; f, "{output:>9}")?;
        } else {
            cwrite!(bright_green; f, "{output:>9}")?;
        }

        Ok(())
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
            let suffix_bounds = min_bound .. max_bound;

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
        if !entry.data.is_file() {
            return self.show_aligned(f, "- -  ", true);
        }

        let bytes = entry.data.len();

        if self.human_readable {
            self.show_human_readable(f, bytes)
        } else {
            self.show_aligned(f, bytes, false)
        }
    }
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
        <_ as ::owo_colors::OwoColorize>::if_supports_color(
            &$display,
            ::owo_colors::Stream::Stdout,
            |v| <_ as ::owo_colors::OwoColorize>::$color(v),
        )
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
        ::std::write!($writer, "{}", $crate::color!($color; ::std::format_args!($($args)+)))
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
        ::std::writeln!($writer, "{}", $crate::color!($color; ::std::format_args!($($args)+)))
    };
}