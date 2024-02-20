#!/bin/sh
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

# Example:
#PIPER_BIN="../piper/install/piper" MODEL="../piper/de_DE-thorsten-high.onnx" ./tools/run-piper.sh text/DE_welcome_usage

set -x

PIPER_BIN:="${PIPER_BIN:-piper}"
MODEL="${MODEL:-de_DE-thorsten-high.onnx}"
INPUT_FILE=$1
OUT_FILE="$(basename $INPUT_FILE).wav"

cat $INPUT_FILE | $PIPER_BIN -m $MODEL --length_scale 1.05 --sentence_silence 0.15 -f tmp-$OUT_FILE
ffmpeg -y -i tmp-$OUT_FILE -ar 48000 -sample_fmt s16 -ac 2 $OUT_FILE

rm tmp-$OUT_FILE

# check file type
# ffmpeg -f ffmetadata - -i $OUT_FILE
# playback
#aplay $OUT_FILE
