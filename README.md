# rs

A Rust implementation of [`ls`](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/ls.html).

Not intended to be POSIX-compliant or an otherwise legimately used tool, just a fun project. See [eza](https://github.com/eza-community/eza) if you're looking for a replacement.

## Usage

```
$ rs [OPTIONS] [PATH]
```
Arguments:
- Path: The path to list (default: `.`)
- Options:
    - `-A`, `--all`: Displays hidden entries
    - `-r`, `--reverse`: Reverses the sorting order
    - `-s`, `--sort` `<SORT_BY>`:
    - Sorts entries using the given method (default: `name`). Possible values:
        - `name`: Sort by name
        - `size`: Sort by size
        - `created`: Sort by creation date
        - `modified`: Sort by last modified
    - `-H`, `--hoist` `<HOIST_BY>`: Groups entries at the top of the listing by the given type (default: `none`). Possible values:
        - `none`: Don't hoist anything
        - `directories`: Hoist directories
        - `hidden`: Hoist hidden files
    - `-U`, `--human-readable`: Whether to use human-readable units
    - `-L`, `--resolve-symlinks`: Resolves symlink paths
    - `-S`, `--show-sizes`: Displays file sizes
    - `-M`, `--show-modified`: Displays file modification dates
    - `-h`, `--help`: Print help (see a summary with `-h`)
    - `-V`, `--version`: Print version

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

rs is licensed under the GNU Affero General Public License version 3, or (at your option) any later version. You should have received a copy of the GNU Affero General Public License along with rs, found in [LICENSE](./LICENSE). If not, see <[https://www.gnu.org/licenses/](https://www.gnu.org/licenses/)>.
