"""Export one exact attacker-turn depth as an xz-compressed absolute uint64 array."""
import os
import pathlib
import struct
import subprocess
import sys


def main():
    if len(sys.argv) != 5:
        raise SystemExit("usage: export_depth.py normal|fairy depth layer.txt.xz result.bin.xz")
    mode, depth_text, source, destination = sys.argv[1:]
    depth = int(depth_text)
    if mode not in ("normal", "fairy") or depth < 1 or depth % 2 != 1:
        raise ValueError("invalid mode or depth")
    destination = pathlib.Path(destination)
    destination.parent.mkdir(parents=True, exist_ok=True)
    decoder = subprocess.Popen(["xz", "-dc", source], stdout=subprocess.PIPE)
    sorter = subprocess.Popen(
        ["sort", "-S", "256M", "-T", str(destination.parent), "-k1,1"],
        env={**os.environ, "LC_ALL": "C"},
        stdin=subprocess.PIPE, stdout=subprocess.PIPE,
    )
    try:
        for line in decoder.stdout:
            fields = line.split()
            if len(fields) != 2 or len(fields[0]) != 16:
                raise ValueError("bad source row")
            key = int(fields[0], 16)
            if key >= 1 << 61:
                raise ValueError("bad source key")
            if int(fields[1]) == depth and key < 1 << 60:
                sorter.stdin.write(fields[0] + b"\n")
    finally:
        decoder.stdout.close()
        sorter.stdin.close()
    if decoder.wait() != 0:
        raise RuntimeError("input xz failed")
    with destination.open("wb") as output:
        compressor = subprocess.Popen(
            ["xz", "-T1", "-6", "-c"], stdin=subprocess.PIPE, stdout=output
        )
        previous = -1
        count = 0
        try:
            for line in sorter.stdout:
                key = int(line, 16)
                if not previous < key < (1 << 60):
                    raise ValueError("duplicate, unsorted, or invalid key")
                compressor.stdin.write(struct.pack("<Q", key))
                previous = key
                count += 1
        finally:
            sorter.stdout.close()
            compressor.stdin.close()
        if sorter.wait() != 0 or compressor.wait() != 0:
            raise RuntimeError("sort or xz failed")
    if subprocess.run(["xz", "-t", str(destination)], check=False).returncode:
        raise RuntimeError("output xz failed validation")
    print(f"{mode}-{depth}: {count} positions", file=sys.stderr)


if __name__ == "__main__":
    main()
