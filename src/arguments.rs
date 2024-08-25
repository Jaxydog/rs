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

use std::path::Path;

use getargs::{Opt, Options};

use crate::sort::{HoistType, SortType};

/// The help command string.
const HELP: &str = concat!(
    env!("CARGO_PKG_DESCRIPTION"),
    "\n\nUsage: ",
    env!("CARGO_BIN_NAME"),
    " [OPTIONS] [PATH]",
    "\n\nOptions:
    -h, --help              Display a list of commands.
    -V, --version           Display this program's version.

    -A, --all               Display hidden files.
    -S, --show-sizes        Display file sizes.
    -P, --show-permissions  Display file permissions.
    -L, --resolve-symlinks  Display resolved symbolic links.

    -r, --reverse           Reverse the displayed sorting order.
    -s, --sort              Sort displayed entries in the specified order.
                            - Default value: name
                            - Possible options: [name, size, created, modified]

    -H, --hoist             Group specific entries at the top of the listing.
                            - Default value: none
                            - Possible options: [none, directories, hidden]

    -U, --human-readable    Display human-readable file sizes.
                            - Only valid when `--show-permissions` is specified"
);

/// The application's command-line arguments.
#[allow(clippy::struct_excessive_bools)]
#[non_exhaustive]
#[derive(Clone, Debug, Default)]
pub struct Arguments {
    /// The directory to list.
    pub path: Option<Box<Path>>,

    /// Whether to display hidden entries.
    pub show_hidden: bool,
    /// Whether to display file sizes.
    pub show_sizes: bool,
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

    /// Whether to use human-readable sizes.
    pub use_human_sizes: bool,
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

    match self::parse_arguments(&Options::new(&arguments)) {
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
fn parse_arguments(options: &Options<'_, String>) -> Output {
    let mut arguments = Arguments::default();

    while let Some(option) = options.next() {
        let option = match option {
            Ok(option) => option,
            Err(error) => return Output::Error(format!("{error}")),
        };

        match option {
            Opt::Long("help") | Opt::Short('h') => {
                println!("{HELP}");

                return Output::Exit;
            }
            Opt::Long("version") | Opt::Short('V') => {
                println!("{}", env!("CARGO_PKG_VERSION"));

                return Output::Exit;
            }
            Opt::Long("all") | Opt::Short('A') => {
                arguments.show_hidden = true;
            }
            Opt::Long("show-sizes") | Opt::Short('S') => {
                arguments.show_sizes = true;
            }
            Opt::Long("show-permissions") | Opt::Short('P') => {
                arguments.show_permissions = true;
            }
            Opt::Long("resolve-symlinks") | Opt::Short('L') => {
                arguments.show_symlinks = true;
            }
            Opt::Long("reverse") | Opt::Short('r') => {
                arguments.sort_reversed = true;
            }
            Opt::Long("sort") | Opt::Short('s') => {
                arguments.sort_function = match options.value_str() {
                    Err(_) | Ok("name") => SortType::Name,
                    Ok("size") => SortType::Size,
                    Ok("created") => SortType::Created,
                    Ok("modified") => SortType::Modified,
                    Ok(other) => return Output::Error(format!("unknown sorting type: {other}")),
                };
            }
            Opt::Long("hoist") | Opt::Short('H') => {
                arguments.hoist_function = match options.value_str() {
                    Err(_) | Ok("none") => HoistType::None,
                    Ok("directories" | "dirs") => HoistType::Directories,
                    Ok("hidden") => HoistType::Hidden,
                    Ok(other) => return Output::Error(format!("unknown hoisting type: {other}")),
                };
            }
            Opt::Long("human-readable") | Opt::Short('U') => {
                arguments.use_human_sizes = true;
            }
            other => return Output::Error(format!("unknown argument: '{other}'")),
        };
    }

    Output::Arguments(arguments)
}