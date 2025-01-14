#!/bin/sh
# SPDX-FileCopyrightText: OpenTalk GmbH <mail@opentalk.eu>
#
# SPDX-License-Identifier: EUPL-1.2

# Example:
#PIPER_BIN="../piper/install/piper"./tools/run-piper.sh -m ../piper/de_DE-thorsten-high.onnx -i text/DE_welcome_usage -o audio/DE_welcome_usage.wav

POSITIONAL_ARGS=()

while [[ $# -gt 0 ]]; do
  case $1 in
    -i|--input)
      INPUT="$2"
      shift # past argument
      shift # past value
      ;;
    -o|--out-file)
      OUTPUT="$2"
      shift # past argument
      shift # past value
      ;;
    -m|--model)
      MODEL="$2"
      shift # past argument
      shift # past value
      ;;
    -p|--playback)
      PLAYBACK="1"
      shift # past argument
      ;;
    -*|--*)
      echo "Unknown option $1"
      exit 1
      ;;
    *)
      POSITIONAL_ARGS+=("$1") # save positional arg
      shift # past argument
      ;;
  esac
done

set -- "${POSITIONAL_ARGS[@]}" # restore positional parameters

echo "INPUT TEXT FILE    = ${INPUT}"
echo "OUTPUT AUDIO FILE  = ${OUTPUT}"
echo "MODEL FILE         = ${MODEL}"
echo "PLAYBACK AUDIO     = ${PLAYBACK:-0}"


if [ ! -f "$MODEL" ]; then
    echo "Model file '$MODEL' does exists."
    exit 1
fi

if [ ! -f "$INPUT" ]; then
    echo "Input text file '$INPUT' does exists."
    exit 1
fi

# set -x

PIPER_BIN="${PIPER_BIN:-piper}"

OUT_FILE="$(basename ${OUTPUT})"
OUT_FILE="${OUT_FILE:-$(basename ${INPUT}).wav}"
OUT_DIR="$(dirname ${OUTPUT})"
OUT_DIR="${OUT_DIR:-$(dirname ${INPUT})}"

TMP_FILE="$(mktemp)"

#cat $INPUT | $PIPER_BIN -m $MODEL --length_scale 1.05 --sentence_silence 0.15  --noise_w 0.8 --noise_scale 0.667 -f $TMP_FILE
cat $INPUT | $PIPER_BIN -m $MODEL --length_scale 1.1 --sentence_silence 0.17  --noise_w 0.8 --noise_scale 0.667 -f $TMP_FILE
ffmpeg -y -i $TMP_FILE -ar 48000 -sample_fmt s16 -ac 2 $OUT_DIR/$OUT_FILE

rm $TMP_FILE

# check file type
# ffmpeg -f ffmetadata - -i $OUT_DIR/$OUT_FILE

if [ "$PLAYBACK" ]; then
    aplay $OUT_DIR/$OUT_FILE
fi
