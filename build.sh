#!/usr/bin/env bash

# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Copyright © 2024 Jaxydog
# Copyright © 2024 RemasteredArch
#
# This file is part of rs.
#
# rs is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
#
# rs is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more details.
#
# You should have received a copy of the GNU Affero General Public License along with rs. If not, see <https://www.gnu.org/licenses/>.

set -euo pipefail # Quit upon any error or attempt to access unset variables

text() {
    local color_name="$1"
    local color=""

    case $color_name in
        bold )
            color="\e[1m"
            ;;
        dim | faint )
            color="\e[2m"
            ;;
        red )
            color="\e[31m"
            ;;
        reset | normal | * )
            color="\e[0m"
            ;;
    esac

    echo -e "$color"
}

error() {
    echo "$(text red)$*$(text reset)" >&2 # Prints to stderr
}

opt() {
    echo "$(text faint)$1$(text reset) | $(text faint)$2$(text reset)"
}

help() {
    cat << EOF
$(text bold)build.sh$(text reset): a build script for rs, a Rust replacement for ls(1).

$(text bold)Arguments:$(text reset)
  $(opt -h --help)     Displays this help message.
  $(opt -i --install)  Installs to ~/.local/bin, or, if that doesn't exist,
                     the current working directory.
  $(opt -c --clean)    Runs \`cargo clean\` before building.
EOF
}

args=""
args=$(getopt \
    --name "build.sh" \
    --options i,c,h \
    --longoptions install,clean,help \
    -- "$@")

eval set -- "$args"
unset args

declare -A opts
opts[install]=false
opts[clean]=false

while true; do
    case "$1" in
        -i | --install )
            opts[install]=true
            shift
            ;;
        -c | --clean )
            opts[clean]=true
            shift
            ;;
        -h | --help )
            help
            exit 0
            ;;
        -- )
            shift
            break
            ;;
        * )
            break
            ;;
    esac
done

if ! grep --quiet 'name = "rs"' './Cargo.toml' 2> /dev/null; then
    error 'Script must be run in root directory of rs!'

    exit 1
fi

if [ -d ./target/ ] && [ "${opts[clean]}" = true ]; then
    echo 'Cleaning up target directory.'

    cargo clean

    echo
fi

echo 'Building command binary.'

cargo build --release

echo

if [ ! -f ./target/release/rs ]; then
    error "Unable to find 'rs' binary."

    exit 1
fi

if [ -d ~/.local/bin/ ] && [ "${opts[install]}" = true ]; then
    target="$HOME/.local/bin/rs"
    [ -f "$target" ] && rm "$target"

    cp './target/release/rs' "$target"

    echo "Compiled and copied to local programs ($target)."
else
    target="./rs"
    [ -f "$target" ] && rm "$target"

    cp './target/release/rs' "$target"

    echo "Compiled and copied to current directory ($target)."
fi

# spiders 🕷️🕸️
