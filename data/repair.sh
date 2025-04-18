#!/bin/bash

rm -rf /tmp/wavs/repaired.wav

dd if=data/working.wav of=/tmp/wavs/header.bin bs=1 count=44
dd if=/tmp/wavs/damaged.wav of=/tmp/wavs/data.bin

data_size=$(stat -c%s /tmp/wavs/data.bin)
chunk_size=$((data_size + 36))
subchunk2_size=$data_size

perl -e 'print pack("V", '$chunk_size')' | dd of=/tmp/wavs/header.bin bs=1 seek=4 conv=notrunc
perl -e 'print pack("V", '$subchunk2_size')' | dd of=/tmp/wavs/header.bin bs=1 seek=40 conv=notrunc

cat /tmp/wavs/header.bin /tmp/wavs/data.bin > /tmp/wavs/repaired_tmp.wav
sox /tmp/wavs/repaired_tmp.wav /tmp/wavs/repaired.wav

rm -rf /tmp/wavs/header.bin /tmp/wavs/data.bin /tmp/wavs/repaired_tmp.wav
