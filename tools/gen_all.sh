#!/bin/sh
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2


# Example
# MODEL_DE=~/dev/piper/de_DE-thorsten-high.onnx MODEL_EN=~/dev/piper/en_GB-cori-high.onnx ../tools/gen_all.sh


SRC_ROOT=$(git rev-parse --show-toplevel)
SCRIPT="$SRC_ROOT/tools/run-piper.sh"

TEXT_DIR="$SRC_ROOT/text"

ARGS="-p" # playback results
set -x

for f in $( ls -t $TEXT_DIR/DE_* ); do \
    NAME=$(basename $f)
    $SCRIPT -m $MODEL_DE -i $f -o $SRC_ROOT/audio/$NAME.wav $ARGS
done

for f in $( ls -t $TEXT_DIR/EN_* ); do \
    NAME=$(basename $f)
    $SCRIPT -m $MODEL_EN -i $f -o $SRC_ROOT/audio/$NAME.wav $ARGS
done
