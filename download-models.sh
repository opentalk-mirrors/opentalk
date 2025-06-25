#!/usr/bin/env bash

download() {
  if ! test -f "$1"; then
    wget "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/$1"
  fi  
}

download ggml-large-v3.bin
download ggml-large-v3-turbo.bin
download ggml-large-v3-turbo-q8_0.bin
