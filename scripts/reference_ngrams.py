#!/usr/bin/env python3
"""Independent overlapping-block fixtures from shifted columns and exact fractions."""

import argparse
from collections import Counter
from fractions import Fraction
from itertools import product
from pathlib import Path


def fixtures() -> tuple[str, int]:
    # zip shifted columns rather than Rust's slice-window iterator / tree tally.
    samples = [bytes(bits) for size in range(6) for bits in product((0, 1), repeat=size)]
    samples += [b"ABABA", b"AAAA", b"AABB", b"ABAB", b"ABCABCABC",
                bytes([255, 0, 128, 255, 0]), bytes(range(256))]
    rows = ["# input_hex\tn\tordered_blocks_hex\tblock_hex:count:exact_probability"]
    for data in samples:
        orders = range(1, len(data) + 3) if len(data) < 256 else [1, 2, 3, 255, 256, 257]
        for n in orders:
            blocks = [bytes(column) for column in zip(*(data[offset:] for offset in range(n)))]
            counts = Counter(blocks)
            entries = []
            for block, count in sorted(counts.items()):
                p = Fraction(count, len(blocks))
                entries.append(f"{block.hex()}:{count}:{p.numerator}/{p.denominator}")
            rows.append("\t".join([data.hex() or "-", str(n),
                                    ",".join(block.hex() for block in blocks) or "-",
                                    ",".join(entries) or "-"]))
    return "\n".join(rows) + "\n", len(rows) - 1


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="verify fixtures without writing")
    args = parser.parse_args()
    path = Path(__file__).resolve().parents[1] / "tests" / "data" / "ngrams.tsv"
    expected, count = fixtures()
    if args.check:
        if path.read_text() != expected:
            raise SystemExit("N-gram reference fixtures differ; regenerate and review them.")
        print(f"{count} independent exact n-gram fixtures verified.")
    else:
        path.write_text(expected)
        print(f"Wrote {count} fixtures to {path}")
