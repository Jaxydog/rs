# rs

A Rust implementation of [`ls`](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/ls.html).

Not intended to POSIX-compliant or an otherwise legimately used tool, just a fun project. See [eza](https://github.com/eza-community/eza) if you're looking for a replacement.

## Features

Currently, rs only implements basic directory listing:
```
$ arch @ PC > ~/dev/rust/rs > ./rs
[drwxr-xr-x]         - .git/
[drwxr-xr-x]         - src/
[drwxr-xr-x]         - target/
[-rw-r--r--]     8   B .gitignore
[-rwxr-xr-x]   2.9 KiB build.sh*
[-rw-r--r--]   3.9 KiB Cargo.lock
[-rw-r--r--]   160   B Cargo.toml
[-rw-r--r--]  33.7 KiB LICENSE
[-rw-r--r--]   1.1 KiB README.md
[lrwxrwxrwx]         - rs -> rs*
[-rw-r--r--]    32   B rust-toolchain.toml
[-rw-r--r--]   1.0 KiB rustfmt.toml
```
*Note that the above contains color, so long as the terminal supports it.*

## License

rs is licensed under the GNU Affero General Public License version 3, or (at your option) any later version. You should have received a copy of the GNU Affero General Public License along with rs, found in [LICENSE](./LICENSE). If not, see <[https://www.gnu.org/licenses/](https://www.gnu.org/licenses/)>.
