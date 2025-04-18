#!/bin/bash

rm -rf test_stuff/

mkdir data/
mkdir test_stuff/

cd data/
wget https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin
cd ../

cd test_stuff/
wget https://github.com/ggerganov/whisper.cpp/raw/master/samples/jfk.wav
cd ../