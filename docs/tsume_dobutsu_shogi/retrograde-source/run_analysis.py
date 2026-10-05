"""Resume retrograde analysis and export every exact odd depth up to the target."""
import lzma
import pathlib
import subprocess
import sys
import time


def run(mode, target):
    if mode not in ("normal", "fairy"):
        raise ValueError("mode must be normal or fairy")
    if target == "all":
        limit = None
    else:
        limit = int(target)
        if limit < 1 or limit % 2 != 1:
            raise ValueError("target depth must be a positive odd number")
    layers = pathlib.Path("build") / f"{mode}-layers"
    layers.mkdir(parents=True, exist_ok=True)
    started = time.monotonic()
    depth = 0
    while (layers / f"depth{depth:02}.txt.xz").exists():
        if not (layers / f"depth{depth + 1:02}.txt.xz").exists():
            break
        depth += 1
    completed = False
    if (layers / f"depth{depth:02}.txt.xz").exists():
        with lzma.open(layers / f"depth{depth:02}.txt.xz", "rb") as source:
            completed = source.read(1) == b""
    if not completed and (limit is None or depth < limit):
        with subprocess.Popen(["xz", "-dc", "data/reachable_1.txt.xz"], stdout=subprocess.PIPE) as decoder:
            subprocess.run(
                ["build/tsume-analyze", mode, target, str(layers)],
                stdin=decoder.stdout, check=True,
            )
            decoder.stdout.close()
            if decoder.wait() != 0:
                raise RuntimeError("reachable input decompression failed")
    files = sorted(layers.glob("depth*.txt.xz"))
    depths = [int(path.stem.split(".")[0][5:]) for path in files]
    highest = max(depths)
    if sorted(depths) != list(range(highest + 1)):
        raise RuntimeError("missing or duplicate checkpoint depth")
    if limit is not None and highest < limit:
        if not completed:
            with lzma.open(layers / f"depth{highest:02}.txt.xz", "rb") as source:
                completed = source.read(1) == b""
        if not completed:
            raise RuntimeError(f"analysis stopped at depth {highest} before target {limit}")
    stop = highest if limit is None else min(highest, limit)
    for odd in range(1, stop + 1, 2):
        output = pathlib.Path(f"{mode}-{odd}.bin.xz")
        if output.exists():
            continue
        source = layers / f"depth{odd:02}.txt.xz"
        if not source.exists():
            raise RuntimeError(f"missing checkpoint: {source}")
        pending = output.with_name(output.name + ".pending")
        subprocess.run([sys.executable, "export_depth.py", mode, str(odd), str(source), str(pending)], check=True)
        pending.replace(output)
        print(f"exported {output}, elapsed {time.monotonic() - started:.1f}s", flush=True)
    if limit is not None and limit > highest:
        for odd in range(highest + 1, limit + 1):
            if odd % 2 == 0:
                continue
            output = pathlib.Path(f"{mode}-{odd}.bin.xz")
            if not output.exists():
                with lzma.open(output, "wb"):
                    pass
                print(f"exported empty {output}", flush=True)
    print(f"{mode}: analyzed through depth {highest}, elapsed {time.monotonic() - started:.1f}s", flush=True)


if __name__ == "__main__":
    if len(sys.argv) != 3:
        raise SystemExit("usage: run_analysis.py normal|fairy odd_depth|all")
    run(sys.argv[1], sys.argv[2])
