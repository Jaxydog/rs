#!/usr/bin/env bash

# SPDX-License-Identifier: AGPL-3.0-or-later
#
# Copyright © 2024 Jaxydog
#
# This file is part of Scripts.
#
# Scripts is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
#
# Scripts is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU Affero General Public License for more details.
#
# You should have received a copy of the GNU Affero General Public License along with Scripts. If not, see <https://www.gnu.org/licenses/>.

if [ -d ./target/ ]; then
    echo 'Cleaning up target directory.'

    cargo clean || exit $?

    echo
fi

echo 'Building command binary.'

cargo build --release || exit $?

echo

if [ ! -f ./target/release/rs ]; then
    echo "Unable to find 'rs' binary."

    exit 1
fi

if [ -d ~/.local/bin/ ]; then
    cp ./target/release/rs ~/.local/bin/rs

    echo 'Compiled and copied to local programs.'
else
    cp ./target/release/rs .

    echo 'Compiled and copied to current directory.'
fi
