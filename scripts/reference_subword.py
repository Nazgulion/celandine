#!/usr/bin/env python3
"""Exact finite-word reference profiles from independently enumerated substrings."""
import argparse
from itertools import product
from pathlib import Path


def fixtures():
    samples = [bytes(bits) for length in range(8) for bits in product((0, 1), repeat=length)]
    samples += [b"ABABA", b"AAAA", b"AABB", b"ABAB", b"ABCD", b"ABCABCABC",
                bytes([255, 0, 128, 255, 0]), "éé".encode(), bytes(range(256))]
    rows = ["# input_hex\tprofile_n0_through_length_plus_one"]
    for data in samples:
        # Enumerate all start/end pairs once, grouping unique substrings by length.
        factors = {data[start:end] for start in range(len(data) + 1)
                   for end in range(start, len(data) + 1)}
        profile = [sum(len(word) == n for word in factors) for n in range(len(data) + 2)]
        rows.append((data.hex() or "-") + "\t" + ",".join(map(str, profile)))
    return "\n".join(rows) + "\n", len(samples)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    path = Path(__file__).resolve().parents[1] / "tests/data/subword.tsv"
    expected, count = fixtures()
    if args.check:
        if path.read_text() != expected:
            raise SystemExit("Subword fixtures differ; regenerate and review.")
        print(f"Verified {count} exact independent subword profiles.")
    else:
        path.write_text(expected)
        print(f"Wrote {count} subword profiles.")
