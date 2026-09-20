#!/usr/bin/env python3
"""Measure operation heap peaks and fresh-process maximum RSS on Linux."""

import argparse
import csv
import hashlib
import io
import json
from pathlib import Path
import platform
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
MIB = 1024 ** 2
FIELDS = ["operation", "shape", "input_bytes", "ngram_length", "support",
          "peak_heap_bytes", "allocation_calls", "input_checksum"]


def cases(smoke):
    if smoke:
        for operation in ["input", "entropy", "extract", "counts", "probabilities", "reuse"]:
            yield operation, "mixed", 4096, 4
        return
    for shape in ["constant", "uniform", "skewed", "mixed"]:
        for size in [10 * MIB, 100 * MIB]:
            for operation in ["input", "entropy", "extract"]:
                yield operation, shape, size, 4
    for shape in ["constant", "uniform"]:
        for size in [10 * MIB, 100 * MIB]:
            for operation in ["counts", "probabilities", "reuse"]:
                yield operation, shape, size, 2
    for size in [65536, MIB, 4 * MIB]:
        yield "input", "mixed", size, 8
        for operation in ["counts", "probabilities", "reuse"]:
            yield operation, "mixed", size, 8


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--smoke", action="store_true", help="small instrument/workflow check")
    parser.add_argument("--repetitions", type=int, default=3)
    args = parser.parse_args()
    if args.repetitions < 1:
        parser.error("repetitions must be positive")
    if platform.system() != "Linux":
        parser.error("process RSS units and evidence currently target Linux")
    build = subprocess.check_output(["cargo", "bench", "--locked", "--bench", "peak_memory",
                                     "--no-run", "--message-format=json"], cwd=ROOT, text=True)
    messages = [json.loads(line) for line in build.splitlines()]
    binary, = [message["executable"] for message in messages
               if message.get("reason") == "compiler-artifact" and message.get("executable")]
    subprocess.run([binary, "--self-check"], check=True)
    parent = ROOT / "target" / "measurements"
    parent.mkdir(parents=True, exist_ok=True)
    output = Path(tempfile.mkdtemp(prefix="peak-memory-", dir=parent))
    metadata = {
        "host": platform.platform(), "repetitions": args.repetitions, "smoke": args.smoke,
        "rustc": subprocess.check_output(["rustc", "-Vv"], text=True),
        "time": subprocess.check_output(["/usr/bin/time", "--version"], text=True),
        "cpu": subprocess.check_output(["lscpu"], text=True),
        "page_bytes": int(subprocess.check_output(["getconf", "PAGESIZE"], text=True)),
        "commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
        "source_sha256": {str(path.relative_to(ROOT)): hashlib.sha256(path.read_bytes()).hexdigest()
                          for path in sorted((ROOT / "src").rglob("*.rs"))
                          + [ROOT / "benches/peak_memory.rs", ROOT / "benches/support/peak_allocator.rs",
                             ROOT / "Cargo.lock", Path(__file__).resolve()]},
        "commands": [],
    }
    print(f"Results: {output}", flush=True)
    with (output / "memory.csv").open("w", newline="") as stream:
        writer = csv.DictWriter(stream, fieldnames=FIELDS + ["repetition", "max_rss_kib"])
        writer.writeheader()
        for operation, shape, size, n in cases(args.smoke):
            print(f"Measuring {operation} {shape} {size} B n={n}", flush=True)
            for repetition in range(1, args.repetitions + 1):
                rss = output / "rss.txt"
                command = ["/usr/bin/time", "-f", "%M", "-o", str(rss), binary,
                           operation, shape, str(size), str(n)]
                metadata["commands"].append(command)
                result = subprocess.run(command, text=True, capture_output=True, check=True)
                values, = list(csv.reader(io.StringIO(result.stdout)))
                assert len(values) == len(FIELDS)
                row = dict(zip(FIELDS, values))
                if operation in ["input", "entropy", "extract", "reuse"]:
                    assert int(row["peak_heap_bytes"]) == int(row["allocation_calls"]) == 0
                else:
                    assert int(row["support"]) > 0 and int(row["peak_heap_bytes"]) > 0
                row.update(repetition=repetition, max_rss_kib=int(rss.read_text().strip()))
                writer.writerow(row)
                stream.flush()
    (output / "manifest.json").write_text(json.dumps(metadata, indent=2) + "\n")
    print(f"Retained memory measurements in {output / 'memory.csv'}", flush=True)


if __name__ == "__main__":
    main()
