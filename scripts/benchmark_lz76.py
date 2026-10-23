#!/usr/bin/env python3
"""Measure exhaustive LZ76 phrase counts."""

import csv
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
BENCHES = ["lz76"]


def main():
    parent = ROOT / "target" / "measurements"
    parent.mkdir(parents=True, exist_ok=True)
    output = Path(tempfile.mkdtemp(prefix="lz76-", dir=parent))
    env = dict(os.environ, CRITERION_HOME=str(output / "criterion"))
    metadata = {
        "host": platform.platform(),
        "rustc": subprocess.check_output(["rustc", "-Vv"], text=True),
        "cpu": subprocess.check_output(["lscpu"], text=True),
        "commit": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
        "rustflags": os.environ.get("RUSTFLAGS", ""),
        "commands": [],
        "source_sha256": {str(path.relative_to(ROOT)): hashlib.sha256(path.read_bytes()).hexdigest()
                          for path in sorted((ROOT / "src").rglob("*.rs"))
                          + [ROOT / "benches" / (name + ".rs") for name in BENCHES]
                          + [ROOT / "Cargo.toml", ROOT / "Cargo.lock", ROOT / "docs/complexity/lz76.md", Path(__file__).resolve()]},
    }
    print(f"Results: {output}", flush=True)
    for name in BENCHES:
        command = ["cargo", "bench", "--locked", "--bench", name, "--",
                   "--sample-size", "10",
                   "--warm-up-time", "0.1", "--measurement-time", "0.5", "--noplot"]
        metadata["commands"].append(command)
        print(f"Measuring {name}", flush=True)
        with (output / (name + ".log")).open("w") as log:
            subprocess.run(command, cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT, check=True)
    rows = []
    for path in sorted((output / "criterion").glob("**/new/estimates.json")):
        benchmark = json.loads(path.with_name("benchmark.json").read_text())
        estimates = json.loads(path.read_text())
        kind = "slope" if estimates.get("slope") else "mean"
        estimate = estimates[kind]
        size = int(benchmark["value_str"]) if benchmark["value_str"] else 0
        ns = estimate["point_estimate"]
        rows.append({"benchmark": benchmark["group_id"], "shape": benchmark["function_id"],
                     "bytes_per_input": size, "estimate": kind, "latency_ns": ns,
                     "ci95_lower_ns": estimate["confidence_interval"]["lower_bound"],
                     "ci95_upper_ns": estimate["confidence_interval"]["upper_bound"],
                     "throughput_mib_s": size / (1024 ** 2) * 1e9 / ns if size else ""})
    # Three periodic shapes at ten sizes; mixed inputs at seven sizes.
    assert len(rows) == 37, len(rows)
    with (output / "latency.csv").open("w", newline="") as stream:
        writer = csv.DictWriter(stream, fieldnames=list(rows[0]))
        writer.writeheader()
        writer.writerows(rows)
    (output / "manifest.json").write_text(json.dumps(metadata, indent=2) + "\n")
    print(f"Retained {len(rows)} measurements in {output / 'latency.csv'}", flush=True)


if __name__ == "__main__":
    main()
