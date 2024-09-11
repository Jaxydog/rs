# rs

A Rust implementation of [`ls`](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/ls.html).

Not intended to be POSIX-compliant or an otherwise legimately used tool, just a fun project. See [eza](https://github.com/eza-community/eza) if you're looking for a replacement.

## Usage

```
$ rs [OPTIONS] [PATH...]
```

Arguments:

- `PATH` - The path(s) to list.

Options:

- `-h`, `--help` - Show the command's usage.

- `-V`, `--version` - Show the command's version.

- `-A`, `--all` - Display hidden files (excluding `.` and `..`)

- `-P`, `--show-permissions` - Display entry permissions.

- `-S`, `--show-sizes` - Display file sizes.

- `-M`, `--show-modified` - Display entry modification date.

- `-L`, `--resolve-symlinks` - Display resolved symbolic links.

- `-r`, `--reverse` - Reverse the displayed sorting order.

- `-s`, `--sort` - Sort displayed entries in the specified order.

  - `name` (default) - Sort by name, descending alphabetically.
  - `size` - Sort by size, descending.
  - `created` - Sort by creation date, descending.
  - `modified` - Sort by modification date, descending.

- `-H`, `--hoist` - Group specific entries at the top of the listing.

  - `none` (default) - Do not hoist any entries.
  - `directories`, `dirs` - Group directories at the top.
  - `hidden` - Group hidden entries at the top.
  - `symlinks` - Group symbolic links at the top.

- `-c`, `--color` - Set whether to use color in the program's output.

- `-U`, `--human-readable` - Use more human-readable formats.

### Examples

Without any options:

```
$ ./rs
build.sh*
Cargo.lock
Cargo.toml
LICENSE
README.md
rs
rust-toolchain.toml
rustfmt.toml
src/
target/
```

With some options:

```
$ ./rs --all --hoist directories --show-sizes --human-readable --resolve-symlinks --sort size
    - -   .git/
    - -   target/
    - -   src/
 33.7 KiB LICENSE
  9.5 KiB Cargo.lock
  2.9 KiB build.sh*
  2.1 KiB README.md
  1.0 KiB rustfmt.toml
  478 B   Cargo.toml
   32 B   rust-toolchain.toml
   19 B   rs -> ././target/release/rs*
    8 B   .gitignore
```

*Note that the above examples contains color, so long as the terminal supports it.*

## License

rs is licensed under the GNU Affero General Public License version 3, or (at your option) any later version. You should have received a copy of the GNU Affero General Public License along with rs, found in [LICENSE](./LICENSE). If not, see \<[https://www.gnu.org/licenses/](https://www.gnu.org/licenses/)>.
