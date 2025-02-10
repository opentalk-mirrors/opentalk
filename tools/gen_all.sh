#!/bin/sh
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2


# Example
# MODEL_DE=~/dev/piper/de_DE-thorsten-high.onnx MODEL_EN=~/dev/piper/en_GB-cori-high.onnx ../tools/gen_all.sh


SRC_ROOT=$(git rev-parse --show-toplevel)
SCRIPT="$SRC_ROOT/tools/run-piper.sh"

TEXT_DIR="$SRC_ROOT/text"

# ARGS="-p" # playback results
# set -x

mkdir -p $SRC_ROOT/audio/de $SRC_ROOT/audio/en

for f in $( ls -t $TEXT_DIR/de/* ); do \
    NAME=$(basename $f)
    $SCRIPT -m $MODEL_DE -i $TEXT_DIR/de/$NAME -o $SRC_ROOT/audio/de/$NAME.wav $ARGS
done

for f in $( ls -t $TEXT_DIR/en/* ); do \
    NAME=$(basename $f)
    $SCRIPT -m $MODEL_EN -i $TEXT_DIR/en/$NAME -o $SRC_ROOT/audio/en/$NAME.wav $ARGS
done
