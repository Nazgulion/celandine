#!/usr/bin/env python3
"""Run bounded, single-worker ASan fuzz campaigns from fresh committed seeds."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
TOOLCHAIN = "nightly-2025-10-25"
TARGETS = {"byte_entropy": 4112, "count_entropy": 2064, "ngrams": 520}


def positive(value):
    number = int(value)
    if number <= 0:
        raise argparse.ArgumentTypeError("must be positive")
    return number


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--runs", type=positive, default=100_000, help="executions per target")
    parser.add_argument("--seed", type=positive, default=20260920)
    parser.add_argument("--target", choices=TARGETS, action="append", help="default: all targets")
    args = parser.parse_args()
    env = dict(os.environ, RUSTUP_TOOLCHAIN=TOOLCHAIN, CARGO_NET_OFFLINE="true")
    # Fetch explicitly during setup; fail rather than silently resolve a changed manifest.
    subprocess.run(["cargo", "metadata", "--manifest-path", "fuzz/Cargo.toml", "--locked",
                    "--format-version", "1"], cwd=ROOT, env=env, stdout=subprocess.DEVNULL, check=True)
    versions = {
        tool: subprocess.check_output(command, cwd=ROOT, env=env, text=True).strip()
        for tool, command in {
            "rustc": ["rustc", "-Vv"], "cargo_fuzz": ["cargo", "fuzz", "--version"],
        }.items()
    }
    if versions["cargo_fuzz"] != "cargo-fuzz 0.13.1":
        raise SystemExit("Install cargo-fuzz 0.13.1 to match the recorded campaign")
    directory = ROOT / "fuzz" / "runs"
    directory.mkdir(parents=True, exist_ok=True)
    run = Path(tempfile.mkdtemp(prefix="campaign-", dir=directory))
    metadata = {"versions": versions, "runs_per_target": args.runs, "seed": args.seed,
                "targets": {}, "commit": subprocess.check_output(
                    ["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip()}
    sources = sorted((ROOT / "src").rglob("*.rs")) + sorted((ROOT / "fuzz").glob("src/*.rs"))
    sources += sorted((ROOT / "fuzz").glob("fuzz_targets/*.rs"))
    sources += [ROOT / path for path in ["fuzz/Cargo.toml", "fuzz/Cargo.lock",
                                        "scripts/fuzz_campaign.py", "scripts/fuzz_seeds.py"]]
    metadata["source_sha256"] = {str(path.relative_to(ROOT)): hashlib.sha256(path.read_bytes()).hexdigest()
                                 for path in sources}
    print(f"Campaign outputs: {run}", flush=True)
    for target in args.target or TARGETS:
        corpus = run / target
        shutil.copytree(ROOT / "fuzz" / "seeds" / target, corpus)
        hashes = {path.name: hashlib.sha256(path.read_bytes()).hexdigest()
                  for path in sorted(corpus.iterdir())}
        artifacts = run / (target + "-artifacts")
        artifacts.mkdir()
        command = ["cargo", "fuzz", "run", target, str(corpus), "--",
                   f"-seed={args.seed}", f"-runs={args.runs}", f"-max_len={TARGETS[target]}",
                   "-timeout=10", "-rss_limit_mb=2048", f"-artifact_prefix={artifacts}/"]
        entry = {"command": command, "starting_corpus_sha256": hashes}
        metadata["targets"][target] = entry
        print(f"Running {target}: {args.runs} executions", flush=True)
        with (run / (target + ".log")).open("w") as log:
            result = subprocess.run(command, cwd=ROOT, env=env, stdout=log, stderr=subprocess.STDOUT)
        entry["exit_code"] = result.returncode
        (run / "manifest.json").write_text(json.dumps(metadata, indent=2) + "\n")
        if result.returncode:
            raise SystemExit(f"{target} failed; inspect {run / (target + '.log')} and saved artifacts")
        print(f"Passed {target}; log: {run / (target + '.log')}", flush=True)


if __name__ == "__main__":
    main()
