#!/bin/bash

rm -rf /tmp/wavs/
mkdir /tmp/wavs/

nc -lvp 14678 >> /tmp/wavs/damaged.wav