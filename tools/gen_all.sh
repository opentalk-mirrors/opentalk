#!/bin/sh
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

SRC_ROOT=$(git rev-parse --show-toplevel)
SCRIPT="$SRC_ROOT/tools/run-piper.sh"

TEXT_DIR="$SRC_ROOT/text"
set -x

for f in $( ls -t $TEXT_DIR ); do \
    NAME=$(basename $f)
    $SCRIPT -m $MODEL -i $TEXT_DIR/$f -o $SRC_ROOT/audio/$NAME.wav
done

