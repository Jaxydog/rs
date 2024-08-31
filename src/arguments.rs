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
    cwrite, cwriteln,
    display::HasColor,
    sort::{HoistType, SortType},
};

/// An option to be displayed in the help listing.
type HelpOption<'a> = (Option<char>, &'a str, &'a str, Option<HelpOptionValues<'a>>);
/// A list of values and their default.
type HelpOptionValues<'a> = (&'a str, &'a [&'a str]);

/// The application's command-line arguments.
#[allow(clippy::struct_excessive_bools)]
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
        ($short:expr, $long:literal, $desc:literal, $default:literal, $($value:literal),* $(,)?) => {
            Some(($short, $long, $desc, Some(($default, &[$default, $($value),*]))))
        };
        ($short:expr, $long:literal, $desc:literal $(,)?) => {
            Some(($short, $long, $desc, None))
        };
    }

    const OPTIONS: &[Option<HelpOption<'static>>] = &[
        option!(Some('h'), "help", "Displays this program's usage."),
        option!(Some('V'), "version", "Displays this program's version."),
        None,
        option!(Some('A'), "all", "Display hidden files (excluding . and ..)."),
        option!(Some('P'), "show-permissions", "Display entry permissions."),
        option!(Some('S'), "show-sizes", "Display file sizes."),
        option!(Some('M'), "show-modified", "Display entry modification date."),
        option!(Some('L'), "resolve-symlinks", "Display resolved symbolic links."),
        None,
        option!(Some('r'), "reverse", "Reverse the displayed sorting order."),
        option!(
            Some('s'),
            "sort",
            "Sort displayed entries in the specified order.",
            "name",
            "size",
            "created",
            "modified"
        ),
        None,
        option!(
            Some('H'),
            "hoist",
            "Group specific entries at the top of the listing.",
            "none",
            "directories",
            "dirs",
            "hidden",
            "symlinks"
        ),
        None,
        option!(Some('c'), "color", "Set whether to use color in the program's output.", "auto", "always", "never"),
        option!(Some('U'), "human-readable", "Use more human-readable formats."),
    ];

    if error {
        self::write_help(arguments, &mut std::io::stderr(), OPTIONS)
    } else {
        self::write_help(arguments, &mut std::io::stdout(), OPTIONS)
    }
}

/// Writes a help display into the given formatter.
///
/// # Errors
///
/// This function will return an error if the display failed to be written.
fn write_help<I>(arguments: &Arguments, f: &mut impl Write, options: I) -> Result<()>
where
    I: IntoIterator<Item = &'static Option<HelpOption<'static>>>,
{
    const OPTION_START_SPACING: usize = 2;
    const OPTION_INNER_SPACING: usize = 18;
    const FULL_SPACING: usize = OPTION_START_SPACING + 4 + 2 + OPTION_INNER_SPACING;

    cwriteln!(arguments, italic; f, "{}", env!("CARGO_PKG_DESCRIPTION"))?;

    f.write_all(b"\n")?;

    cwrite!(arguments, bold; f, "Usage:")?;

    f.write_all(concat!(" ", env!("CARGO_PKG_NAME"), " [OPTIONS] [PATH...]\n\n").as_bytes())?;

    cwriteln!(arguments, bold; f, "Options:")?;

    for option in options {
        let Some((short, long, description, values)) = *option else {
            f.write_all(b"\n")?;

            continue;
        };

        f.write_all(&b" ".repeat(OPTION_START_SPACING))?;

        if let Some(short) = short {
            cwrite!(arguments, bright_cyan; f, "-{short}")?;
            f.write_all(b", ")?;
        } else {
            f.write_all(b"    ")?;
        }

        cwrite!(arguments, bright_cyan; f, "--{long}")?;

        let spacing = (OPTION_INNER_SPACING - 1).saturating_sub(long.len()) + 1;

        f.write_all(&b" ".repeat(spacing))?;

        writeln!(f, "{description}")?;

        if let Some((default, values)) = values {
            f.write_all(&b" ".repeat(FULL_SPACING))?;

            cwrite!(arguments, bright_black; f, "-")?;

            f.write_all(b" ")?;

            cwrite!(arguments, italic; f, "Default value:")?;

            f.write_all(b" ")?;

            cwriteln!(arguments, bold; f, "{default}")?;

            if values.is_empty() {
                continue;
            }

            f.write_all(&b" ".repeat(FULL_SPACING))?;

            cwrite!(arguments, bright_black; f, "-")?;

            f.write_all(b" ")?;

            cwrite!(arguments, italic; f, "Possible values:")?;

            f.write_all(b" ")?;

            for (index, value) in values.iter().enumerate() {
                cwrite!(arguments, bold; f, "{value}")?;

                if index < values.len() - 1 {
                    f.write_all(b", ")?;
                }
            }

            f.write_all(b"\n")?;
        }
    }

    Ok(())
}
