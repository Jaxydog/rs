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

use std::{
    io::{Result, Write},
    path::{Path, PathBuf},
};

use getargs::{Arg, Opt, Options};

use crate::{
    display::HasColor,
    sort::{HoistType, SortType},
};

/// An option to be displayed in the help listing.
type HelpOption<'a> = (Option<char>, &'a str, &'a str, Option<HelpOptionValues<'a>>);
/// A list of values and their default.
type HelpOptionValues<'a> = (&'a str, &'a [&'a str]);

/// The application's command-line arguments.
#[expect(clippy::struct_excessive_bools, reason = "a lot of command-line arguments are flags")]
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Arguments {
    /// The directories to list.
    pub paths: Box<[Box<Path>]>,

    /// Whether to display hidden entries.
    pub show_hidden: bool,
    /// Whether to display file sizes.
    pub show_sizes: bool,
    /// Whether to display file modification date.
    pub show_modified: bool,
    /// Whether to display file permissions.
    pub show_permissions: bool,
    /// Whether to display file owners.
    pub show_owner: bool,
    /// Whether to display resolved symbolic links.
    pub show_symlinks: bool,

    /// The method to use to sort the displayed entries.
    pub sort_function: SortType,
    /// Whether to reverse the displayed sorting order.
    pub sort_reversed: bool,

    /// The method to use to hoist the displayed entries.
    pub hoist_function: HoistType,

    /// Whether to use color in the program's output.
    pub color: Option<bool>,
    /// Whether to use human-readable sizes.
    pub human_readable: bool,
}

impl HasColor for Arguments {
    fn has_color(&self) -> Option<bool> {
        self.color
    }
}

/// The output of parsing arguments.
#[derive(Clone, Debug)]
pub enum Output {
    /// The arguments.
    Arguments(Arguments),
    /// Exit early.
    Exit,
    /// An error during parsing.
    Error(String),
}

/// Parses the command-line arguments from standard in.
///
/// This function will exit early if the arguments could not be parsed.
#[must_use]
pub fn parse() -> Arguments {
    let arguments = std::env::args().skip(1).collect::<Box<[_]>>();

    match self::parse_arguments(Options::new(arguments.iter().map(String::as_str))) {
        Output::Arguments(arguments) => arguments,
        Output::Exit => {
            drop(arguments);

            std::process::exit(0);
        }
        Output::Error(error) => {
            eprintln!("{error}");

            drop(arguments);
            drop(error);

            std::process::exit(1);
        }
    }
}

/// Parses the given options.
fn parse_arguments<'arg>(mut options: Options<&'arg str, impl Iterator<Item = &'arg str>>) -> Output {
    let mut arguments = Arguments::default();

    while let Some(option) = options.next_opt().transpose() {
        let option = match option {
            Ok(option) => option,
            Err(error) => return Output::Error(format!("{error}")),
        };

        match option {
            Opt::Long("help") | Opt::Short('h') => {
                self::print_help(&arguments, false).expect("failed to print help menu");

                return Output::Exit;
            }
            Opt::Long("version") | Opt::Short('V') => {
                println!("{}", env!("CARGO_PKG_VERSION"));

                return Output::Exit;
            }
            Opt::Long("all") | Opt::Short('A') => {
                arguments.show_hidden = true;
            }
            Opt::Long("show-permissions") | Opt::Short('P') => {
                arguments.show_permissions = true;
            }
            Opt::Long("show-sizes") | Opt::Short('S') => {
                arguments.show_sizes = true;
            }
            Opt::Long("show-modified") | Opt::Short('M') => {
                arguments.show_modified = true;
            }
            Opt::Long("show-owner") | Opt::Short('O') => {
                arguments.show_owner = true;
            }
            Opt::Long("resolve-symlinks") | Opt::Short('L') => {
                arguments.show_symlinks = true;
            }
            Opt::Long("reverse") | Opt::Short('r') => {
                arguments.sort_reversed = true;
            }
            Opt::Long("sort") | Opt::Short('s') => {
                arguments.sort_function = match options.value() {
                    Err(_) | Ok("name") => SortType::Name,
                    Ok("size") => SortType::Size,
                    Ok("created") => SortType::Created,
                    Ok("modified") => SortType::Modified,
                    Ok(other) => return Output::Error(format!("unknown sorting type: {other}")),
                };
            }
            Opt::Long("hoist") | Opt::Short('H') => {
                arguments.hoist_function = match options.value() {
                    Err(_) | Ok("none") => HoistType::None,
                    Ok("directories" | "dirs") => HoistType::Directories,
                    Ok("hidden") => HoistType::Hidden,
                    Ok("symlinks") => HoistType::Symlinks,
                    Ok(other) => return Output::Error(format!("unknown hoisting type: {other}")),
                };
            }
            Opt::Long("color") | Opt::Short('c') => {
                arguments.color = match options.value() {
                    Err(_) | Ok("auto") => None,
                    Ok("always") => Some(true),
                    Ok("never") => Some(false),
                    Ok(other) => return Output::Error(format!("unknown color choice: {other}")),
                }
            }
            Opt::Long("human-readable") | Opt::Short('U') => {
                arguments.human_readable = true;
            }
            other => return Output::Error(format!("unknown argument: '{other}'")),
        };
    }

    let mut paths = Vec::with_capacity(1);

    while let Ok(Some(Arg::Positional(path))) = options.next_arg() {
        paths.push(PathBuf::from(path).into_boxed_path());
    }

    arguments.paths = paths.into_boxed_slice();

    Output::Arguments(arguments)
}

/// Prints a help display.
///
/// # Errors
///
/// This function will return an error if the display could not be printed.
fn print_help(arguments: &Arguments, error: bool) -> Result<()> {
    macro_rules! option {
        ($short:literal, $long:literal, $desc:literal, [$default:literal, $($value:literal),* $(,)?]) => {
            Some((Some($short), $long, $desc, Some(($default, &[$default, $($value),*]))))
        };
        ($short:literal, $long:literal, $desc:literal $(,)?) => {
            Some((Some($short), $long, $desc, None))
        };
        ($long:literal, $desc:literal, [$default:literal, $($value:literal),* $(,)?]) => {
            Some((None, $long, $desc, Some(($default, &[$default, $($value),*]))))
        };
        ($long:literal, $desc:literal $(,)?) => {
            Some((None, $long, $desc, None))
        };
    }

    const OPTIONS: &[Option<HelpOption<'static>>] = &[
        option!('h', "help", "Show this program's usage."),
        option!('V', "version", "Show this program's version."),
        None,
        option!('A', "all", "Display hidden files (excluding . and ..)."),
        option!('P', "show-permissions", "Display entry permissions."),
        option!('S', "show-sizes", "Display file sizes."),
        option!('M', "show-modified", "Display entry modification date."),
        option!('O', "show-owner", "Display entry owner."),
        option!('L', "resolve-symlinks", "Display resolved symbolic links."),
        None,
        option!('r', "reverse", "Reverse the displayed sorting order."),
        option!(
            's',
            "sort",
            "Sort displayed entries in the specified order.",
            ["name", "size", "created", "modified"]
        ),
        None,
        option!(
            'H',
            "hoist",
            "Group specific entries at the top of the listing.",
            ["none", "directories", "dirs", "hidden", "symlinks"]
        ),
        None,
        option!('c', "color", "Set whether to use color in the program's output.", ["auto", "always", "never"]),
        option!('U', "human-readable", "Use more human-readable formats."),
    ];

    if error {
        self::write_help(arguments, &mut std::io::stderr(), error, OPTIONS)
    } else {
        self::write_help(arguments, &mut std::io::stdout(), error, OPTIONS)
    }
}

macro_rules! cprint {
    ($error:expr, $self:expr, $color:ident; $write:expr, $($body:tt)*) => {
        if $error {
            $crate::cwrite!($self, $color, ::owo_colors::Stream::Stderr; $write, $($body)*)
        } else {
            $crate::cwrite!($self, $color, ::owo_colors::Stream::Stdout; $write, $($body)*)
        }
    };
}

macro_rules! cprintln {
    ($error:expr, $self:expr, $color:ident; $write:expr, $($body:tt)*) => {
        if $error {
            $crate::cwriteln!($self, $color, ::owo_colors::Stream::Stderr; $write, $($body)*)
        } else {
            $crate::cwriteln!($self, $color, ::owo_colors::Stream::Stdout; $write, $($body)*)
        }
    };
}

/// Writes a help display into the given formatter.
///
/// # Errors
///
/// This function will return an error if the display failed to be written.
fn write_help<I>(arguments: &Arguments, f: &mut impl Write, error: bool, options: I) -> Result<()>
where
    I: IntoIterator<Item = &'static Option<HelpOption<'static>>>,
{
    cprintln!(error, arguments, italic; f, "{}", env!("CARGO_PKG_DESCRIPTION"))?;

    f.write_all(b"\n")?;

    cprint!(error, arguments, bold; f, "Usage:")?;

    f.write_all(concat!(" ", env!("CARGO_PKG_NAME"), " [OPTIONS] [PATH...]\n\n").as_bytes())?;

    cprintln!(error, arguments, bold; f, "Options:")?;

    for option in options {
        if let Some(option) = *option {
            self::write_help_option(arguments, f, error, option)?;
        } else {
            f.write_all(b"\n")?;
        }
    }

    Ok(())
}

/// Writes a help display's option into the given formatter.
///
/// # Errors
///
/// This function will return an error if the display failed to be written.
fn write_help_option(
    arguments: &Arguments,
    f: &mut impl Write,
    error: bool,
    (short, long, description, values): HelpOption<'_>,
) -> Result<()> {
    /// The number of spaces to add to the front of the option listing.
    const START_PAD: &[u8] = b"  ";
    /// The number of spaces to add between the options and their descriptions.
    const GAP_WIDTH: usize = 24;
    /// The total number of characters that the short option takes up.
    const SHORT_LEN: usize = "-A, ".len();
    /// The total number of characters that '--' takes up.
    const LONG_DASH_LEN: usize = "--".len();
    /// The total number of space before the description should be printed.
    const DESCRIPTION_OFFSET: usize = START_PAD.len() + SHORT_LEN + LONG_DASH_LEN + GAP_WIDTH;

    f.write_all(START_PAD)?;

    if let Some(short) = short {
        cprint!(error, arguments, bright_cyan; f, "-{short}")?;

        f.write_all(b", ")?;
    } else {
        f.write_all(&b" ".repeat(SHORT_LEN))?;
    }

    let long_len = long.chars().count();

    // Truncate if the option overflows.
    if long_len >= GAP_WIDTH {
        cprint!(error, arguments, bright_cyan; f, "--{}...", &long[..GAP_WIDTH - 4])?;
    } else {
        cprint!(error, arguments, bright_cyan; f, "--{long}")?;
    }

    // Ensure at least one space is always printed betwixt the options and their descriptions.
    let spacing = (GAP_WIDTH - 1).saturating_sub(long_len) + 1;

    f.write_all(&b" ".repeat(spacing))?;

    writeln!(f, "{description}")?;

    values.map_or(Ok(()), |values| self::write_help_option_values::<DESCRIPTION_OFFSET>(arguments, f, error, values))
}

/// Writes a help display's option's values into the given formatter.
///
/// # Errors
///
/// This function will return an error if the display failed to be written.
fn write_help_option_values<const DESCRIPTION_OFFSET: usize>(
    arguments: &Arguments,
    f: &mut impl Write,
    error: bool,
    (default, values): HelpOptionValues<'_>,
) -> Result<()> {
    f.write_all(&b" ".repeat(DESCRIPTION_OFFSET))?;

    cprint!(error, arguments, bright_black; f, "-")?;

    f.write_all(b" ")?;

    cprint!(error, arguments, italic; f, "Default value:")?;

    f.write_all(b" ")?;

    cprintln!(error, arguments, bold; f, "{default}")?;

    if values.is_empty() {
        return Ok(());
    }

    f.write_all(&b" ".repeat(DESCRIPTION_OFFSET))?;

    cprint!(error, arguments, bright_black; f, "-")?;

    f.write_all(b" ")?;

    cprint!(error, arguments, italic; f, "Possible values:")?;

    f.write_all(b" ")?;

    for (index, value) in values.iter().enumerate() {
        cprint!(error, arguments, bold; f, "{value}")?;

        if index < values.len() - 1 {
            f.write_all(b", ")?;
        }
    }

    f.write_all(b"\n")
}
