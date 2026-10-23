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
        for operation in ["input", "entropy", "extract", "counts", "probabilities", "reuse", "total_variation", "jensen_shannon"]:
            yield operation, "mixed", 4096, 4
        yield "lz76", "mixed", 4096, 1
        yield "subword", "mixed", 4096, 4
        yield "subword_profile", "mixed", 64, 65
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


def subword_cases():
    for shape in ["constant", "uniform", "skewed", "mixed"]:
        for size in [10 * MIB, 100 * MIB]:
            yield "input", shape, size, 1
            yield "subword", shape, size, 1
            if shape in ["constant", "uniform"]:
                yield "subword", shape, size, 2
    for size in [65536, MIB]:
        yield "input", "mixed", size, 8
        yield "subword", "mixed", size, 8
    for shape in ["constant", "mixed"]:
        for size in [64, 1024]:
            yield "input", shape, size, 1
            yield "subword_profile", shape, size, size + 1
    yield "subword_profile", "mixed", 65536, 9
    yield "subword_profile", "mixed", 100 * MIB, 2


def lz76_cases():
    for shape in ["constant", "uniform", "skewed"]:
        for size in [10 * MIB, 100 * MIB]:
            for operation in ["input", "lz76"]:
                yield operation, shape, size, 1
    for size in [4096, 16384, 65536]:
        for operation in ["input", "lz76"]:
            yield operation, "mixed", size, 1


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--smoke", action="store_true", help="small instrument/workflow check")
    parser.add_argument("--total-variation", action="store_true",
                        help="measure only TV and input controls at 10/100 MiB")
    parser.add_argument("--jensen-shannon", action="store_true",
                        help="measure only JS and input controls at 10/100 MiB")
    parser.add_argument("--lz76", action="store_true", help="LZ76 scalar memory with bounded diverse inputs")
    parser.add_argument("--subword", action="store_true", help="subword scalar and bounded profile memory")
    parser.add_argument("--repetitions", type=int, default=3)
    args = parser.parse_args()
    if args.repetitions < 1:
        parser.error("repetitions must be positive")
    if sum([args.smoke, args.total_variation, args.jensen_shannon, args.subword, args.lz76]) > 1:
        parser.error("choose one of --smoke, --total-variation, --jensen-shannon, --subword, or --lz76")
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
        "lz76": args.lz76, "subword": args.subword, "total_variation": args.total_variation, "jensen_shannon": args.jensen_shannon,
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
        comparison = "jensen_shannon" if args.jensen_shannon else "total_variation"
        workloads = ((operation, shape, size, 1)
                     for shape in ["constant", "uniform", "skewed", "mixed"]
                     for size in [10 * MIB, 100 * MIB]
                     for operation in ["input", comparison]) if (args.total_variation or args.jensen_shannon) else cases(args.smoke)
        if args.lz76:
            workloads = lz76_cases()
        if args.subword:
            workloads = subword_cases()
        for operation, shape, size, n in workloads:
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
                no_heap = (operation in ["input", "entropy", "extract", "reuse", "total_variation", "jensen_shannon", "lz76"]
                           or (operation == "subword" and (n <= 1 or n >= size))
                           or (operation == "subword_profile" and (n <= 2 or size <= 2)))
                if no_heap:
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
