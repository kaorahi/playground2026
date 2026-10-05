"""Validate decompressed array structure and strictly increasing absolute keys."""
import lzma
import pathlib
import struct
import sys

for name in sys.argv[1:]:
    previous = -1
    count = 0
    with lzma.open(name, "rb") as source:
        while chunk := source.read(8 * 65536):
            if len(chunk) % 8:
                raise ValueError(f"misaligned raw array: {name}")
            for (key,) in struct.iter_unpack("<Q", chunk):
                if not previous < key < (1 << 60):
                    raise ValueError(f"invalid order or key: {name}")
                previous = key
                count += 1
    print(f"{pathlib.Path(name).name}: {count} positions")
