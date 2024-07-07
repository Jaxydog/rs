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

use std::cmp::Ordering;
use std::env::{args, current_dir};
use std::fs::canonicalize;
use std::io::Write;
#[cfg(unix)] use std::os::unix::fs::MetadataExt;

use owo_colors::{OwoColorize, Stream};

#[cfg(unix)]
display_impl! {
    #[repr(transparent)]
    pub struct ModeDisplay {
        pub mode: u32,
    } => {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", '['.if_supports_color(Stream::Stdout, |v| v.bright_black()))?;

            for character in unix_mode::to_string(self.mode).chars() {
                write!(f, "{}", ModeCharDisplay { character })?;
            }

            write!(f, "{}", ']'.if_supports_color(Stream::Stdout, |v| v.bright_black()))
        }
    }

    #[repr(transparent)]
    pub struct ModeCharDisplay {
        pub character: char,
    } => {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write_by_pattern! {
                use f, self.character;

                '-' => bright_black,
                'd' => bright_blue,
                'r' => bright_yellow,
                'w' => bright_red,
                'x' => bright_green,
                'l' => bright_cyan,
                'b' => bright_magenta,
                'c' => bright_magenta,
                'p' => bright_magenta,
                's' => bright_magenta,
            }
        }
    }
}

display_impl! {
    pub struct FileSizeDisplay<'fp> {
        pub bytes: u64,
        pub metadata: &'fp std::fs::Metadata,
    } => {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            const SUFFIXES: &[&str] = &["  B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB"];

            if !self.metadata.is_file() {
                return write!(f, "{:>9}", '-'.if_supports_color(Stream::Stdout, |v| v.bright_black()));
            } else if self.bytes == 0 {
                return write!(f, "{:>9}", "0   B".if_supports_color(Stream::Stdout, |v| v.bright_green()));
            }

            let mut string = None;

            for (index, suffix) in SUFFIXES.iter().enumerate() {
                let min_bound = 1 << (10 * index);
                let max_bound = 1 << (10 * (index + 1));

                if (min_bound..max_bound).contains(&self.bytes) {
                    if index > 0 {
                        let value = self.bytes as f64 / min_bound as f64;

                        string = Some(format!("{value:.1} {suffix}"));
                    } else {
                        string = Some(format!("{} {suffix}", self.bytes));
                    };

                    break;
                }
            }

            let string = string.unwrap_or_else(|| format!("{}", self.bytes));

            write!(f, "{:>9}", string.if_supports_color(Stream::Stdout, |v| v.bright_green()))
        }
    }

    pub struct FileNameDisplay<'fp> {
        pub path: &'fp std::path::Path,
        pub metadata: &'fp std::fs::Metadata,
    } => {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let name = self.path.file_name().unwrap_or(self.path.as_os_str());
            let name = name.to_string_lossy().into_owned();
            let executable = is_executable(self.metadata);

            write_by_condition! {
                use f;

                self.metadata.is_dir() => format_args!("{name}/"), bright_blue;
                self.metadata.is_symlink() => name, bright_cyan;
                executable => name, bright_green;
                true => name, white;
            }

            if executable {
                write!(f, "*")?;
            }

            if self.metadata.is_symlink() {
                write!(f, "{}", " -> ".if_supports_color(Stream::Stdout, |v| v.bright_black()))?;

                let Ok((Ok(ref metadata), ref path)) = canonicalize(self.path).map(|p| (p.metadata(), p)) else {
                    return Ok(());
                };

                write!(f, "{}", FileNameDisplay { path, metadata })?;
            }

            Ok(())
        }
    }
}

fn is_executable(metadata: &std::fs::Metadata) -> bool {
    metadata.is_file() && if cfg!(unix) { metadata.mode() & 1 != 0 } else { false }
}

fn main() -> std::io::Result<()> {
    let path = args().nth(1).map_or_else(current_dir, canonicalize)?;
    let mut entries: Vec<_> = std::fs::read_dir(path)?.filter_map(Result::ok).collect();

    entries.sort_unstable_by(|a, b| {
        let a_path = a.path().to_string_lossy().to_lowercase();
        let b_path = b.path().to_string_lossy().to_lowercase();
        let order = a_path.cmp(&b_path);

        let (Ok(a_metadata), Ok(b_metadata)) = (a.metadata(), b.metadata()) else {
            return order;
        };

        match (a_metadata.is_dir(), b_metadata.is_dir()) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => Ordering::Equal,
        }
        .then(order)
    });

    let mut lock = std::io::stdout().lock();

    for entry in entries {
        let metadata = &entry.metadata()?;

        #[cfg(unix)]
        write!(&mut lock, "{} ", ModeDisplay { mode: metadata.mode() })?;
        write!(&mut lock, "{} ", FileSizeDisplay { bytes: metadata.size(), metadata })?;
        writeln!(&mut lock, "{}", FileNameDisplay { path: &entry.path(), metadata })?;
    }

    Ok(())
}

#[macro_export]
macro_rules! write_by_condition {
    (use $formatter:expr; $($condition:expr => $display:expr, $color:ident;)*) => {$(
        if $condition {
            write!($formatter, "{}", $display.if_supports_color(Stream::Stdout, |v| v.$color()))?;
        }
    )else*};
}

#[macro_export]
macro_rules! write_by_pattern {
    (use $formatter:expr, $expression:expr; $($pattern:literal => $color:ident),* $(,)?) => {
        match $expression {
            $(character @ $pattern => write!($formatter, "{}", character.if_supports_color(Stream::Stdout, |c| c.$color())),)*
            default => write!($formatter, "{default}"),
        }
    };
    (use $formatter:expr, $expression:expr; $($pattern:expr => $fmt:literal, $color:ident),* $(,)?) => {
        match $expression {
            $(character @ $pattern => write!($formatter, "{}", format_args!($fmt, character).if_supports_color(Stream::Stdout, |c| c.$color())),)*
            default => write!($formatter, "{default}"),
        }
    }
}

#[macro_export]
macro_rules! display_impl {
    ($(
        $(#[$attribute:meta])*
        $visibility:vis struct $type_name:ident
        $(<$($generic:tt),* $(,)?>)?
        $(where $(
            $clause:ident: $initial_bound:tt $(+ $bound:ident)* $(+ $scope:lifetime)*
        ),+ $(,)?)?
        $({$(
            $(#[$field_attribute:meta])*
            $field_visibility:vis $field_name:ident: $field_type:ty
        ),+ $(,)?})? => {$(
            $impl_body:tt
        )*}
    )*) => {$(
        $(#[$attribute])*
        $visibility struct $type_name
        $(<$($generic),*>)?
        $(where $(
            $clause: $initial_bound $(+ $bound)* $(+ $scope)*
        ),+)?
        {$($(
            $(#[$field_attribute])*
            $field_visibility $field_name: $field_type,
        )*)?}

        impl
        $(<$($generic),*>)?
        ::std::fmt::Display for $type_name
        $(<$($generic),*>)?
        $(where $(
            $clause: $initial_bound $(+ $bound)* $(+ $scope)*
        ),+)?
        {$(
            $impl_body
        )*}
    )*};
}
