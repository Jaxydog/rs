use std::cmp::Ordering;
use std::env::{args, current_dir};
use std::fmt::Display;
use std::fs::canonicalize;
use std::io::Write;
#[cfg(target_os = "linux")] use std::os::unix::fs::MetadataExt;

use owo_colors::{OwoColorize, Stream};

fn main() -> std::io::Result<()> {
    let path = args().nth(1).map_or_else(current_dir, canonicalize)?;
    let dir = std::fs::read_dir(path)?;
    let mut entries: Vec<_> = dir.map_while(Result::ok).collect();

    entries.sort_unstable_by(|a, b| {
        let order = a.path().cmp(&b.path());

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
        let metadata = entry.metadata()?;

        #[cfg(target_os = "linux")]
        {
            let mode = unix_mode::to_string(metadata.mode());

            write!(&mut lock, "{} ", display_mode(&mode))?;
        }

        writeln!(&mut lock, "{}", display_file(&entry, &metadata))?;
    }

    Ok(())
}

macro_rules! anonymous_display {
    ($(fn $name:ident $(<$($lt:tt),+ $(,)?>)? ($($argument:ident: $type:ty),* $(,)?) { $($body:tt)* })*) => {$(
        fn $name $(<$($lt),+>)? ($($argument: $type),*) -> impl Display $($(+ $lt)*)? {
            struct Struct $(<$($lt),+>)? ($($type),*);

            impl $(<$($lt),+>)? Display for Struct $(<$($lt),+>)? {
                $($body)*
            }

            Struct($($argument),*)
        }
    )*};
}

macro_rules! colorize_chars {
    ($f:expr, $character:expr, [$($bind:literal -> $color:ident),* $(,)?]) => {
        match $character {
            $(
                c @ $bind => write!($f, "{}", c.if_supports_color(Stream::Stdout, |v| v.$color())),
            )*
            default => write!($f, "{default}"),
        }
    };
}

macro_rules! colorize_if {
    ($($f:expr, $predicate:expr, $fmt:literal, $value:expr, $color:ident);* $(;)?) => {
        $(
            if $predicate {
                write!($f, "{}", format_args!($fmt, $value).if_supports_color(Stream::Stdout, |v| v.$color()))?;
            }
        )else*
    };
}

anonymous_display! {
    fn display_mode<'c>(string: &'c str) {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", '['.if_supports_color(Stream::Stdout, |v| v.bright_black()))?;

            for character in self.0.chars() {
                colorize_chars!(f, character, [
                    '-' -> bright_black,
                    'd' -> bright_blue,
                    'r' -> bright_yellow,
                    'w' -> bright_red,
                    'x' -> bright_green,
                    'l' -> bright_cyan,
                    'b' -> bright_magenta,
                    'c' -> bright_magenta,
                    'p' -> bright_magenta,
                    's' -> bright_magenta,
                ])?;
            }

            write!(f, "{}", ']'.if_supports_color(Stream::Stdout, |v| v.bright_black()))
        }
    }

    fn display_file<'f>(file: &'f std::fs::DirEntry, meta: &'f std::fs::Metadata) {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            fn is_executable(metadata: &std::fs::Metadata) -> bool {
                cfg!(target_os = "linux") && metadata.is_file() && metadata.mode() & 1 != 0
            }

            let name = self.0.file_name().to_string_lossy().into_owned();

            colorize_if! {
                f, self.1.is_dir(), "{}/", name, bright_blue;
                f, self.1.is_symlink(), "{}", name, bright_cyan;
                f, is_executable(self.1), "{}", name, bright_green;
                f, true, "{}", name, white;
            };

            if is_executable(self.1) {
                write!(f, "*")?;
            }

            if self.1.is_symlink() {
                write!(f, " -> ")?;

                let Ok(path) = canonicalize(self.0.path()) else {
                    return Ok(());
                };
                let Ok(path_metadata) = path.metadata() else {
                    return Ok(());
                };

                let name = path.to_string_lossy().into_owned();

                colorize_if! {
                    f, path_metadata.is_dir(), "{}/", name, bright_blue;
                    // \/ Should not be possible.
                    f, path_metadata.is_symlink(), "{}", name, bright_cyan;
                    f, is_executable(&path_metadata), "{}", name, bright_green;
                    f, true, "{}", name, white;
                };

                if is_executable(&path_metadata) {
                    write!(f, "*")?;
                }
            }

            Ok(())
        }
    }
}
