#!/usr/bin/env python3
"""Generate or verify the small, portable libFuzzer starting corpus."""

import argparse
import math
from pathlib import Path
import struct

ROOT = Path(__file__).resolve().parents[1] / "fuzz" / "seeds"


def base_seeds():
    values = [
        ("zero", 0.0), ("negative_zero", -0.0), ("negative", -1.0),
        ("half", 0.5), ("one_below", math.nextafter(1.0, 0.0)),
        ("one", 1.0), ("one_above", math.nextafter(1.0, math.inf)),
        ("two", 2.0), ("ten", 10.0), ("tiny", math.ulp(0.0)),
        ("huge", float.fromhex("0x1.fffffffffffffp+1023")),
        ("infinity", math.inf), ("negative_infinity", -math.inf),
        ("nan", math.nan),
    ]
    maximum = (1 << 64) - 1
    for name, value in values:
        header = struct.pack("<dd", value, value)
        yield "byte_entropy", name, header + b"AAAB"
        yield "count_entropy", name, header + struct.pack("<QQ", maximum - 1, 1)
    header = struct.pack("<dd", 0.5, 2.0)
    for name, data in [
        ("empty", b""), ("constant", b"A" * 64), ("uniform", bytes(range(256))),
        ("maximum_length", bytes(range(256)) * 16), ("binary", b"\x00\xff\x00"),
    ]:
        yield "byte_entropy", name, header + data
    for name, counts in [
        ("empty", []), ("constant", [maximum]), ("balanced", [5] * 256),
        ("overflow_one", [maximum, 1]), ("overflow_many", [maximum] * 256),
        ("exact_maximum", [maximum - 1000, 999, 1]),
    ]:
        yield "count_entropy", name, header + b"".join(struct.pack("<Q", c) for c in counts)
    for name, n, data in [
        ("zero_empty", 0, b""), ("zero_nonempty", 0, b"AB"),
        ("empty", 1, b""), ("single", 1, b"ABABA"),
        ("overlap", 2, b"ABABA"), ("constant", 3, b"A" * 512),
        ("binary", 2, bytes(range(256)) * 2), ("full", 5, b"ABABA"),
        ("oversized", 6, b"ABABA"), ("huge", maximum, b"AB"),
    ]:
        yield "ngrams", name, struct.pack("<Q", n) + data
    for name, left, right in [
        ("worked", b"AAAB", b"AB"), ("equal", b"AABB", b"ABAB"),
        ("disjoint", b"AAA", b"BB"), ("left_empty", b"", b"AB"),
        ("right_empty", b"AB", b""),
        ("maximum_bytes", bytes(range(256)) * 8, bytes(range(255, -1, -1)) * 8),
    ]:
        yield "total_variation", name, b"\0" + struct.pack("<Q", len(left)) + left + right
    for name, pairs in [
        ("tiny", [(maximum - 1, maximum - 2), (1, 1)]),
        ("maximum_disjoint", [(maximum, 0), (0, maximum)]),
        ("scaled", [(3, 9), (1, 3)]), ("count_empty", [(0, 1)]),
        ("overflow", [(maximum, maximum), (1, 1)]),
        ("full_support", [(i + 1, 256 - i) for i in range(256)]),
    ]:
        yield "total_variation", name, b"\1" + b"".join(struct.pack("<QQ", a, b) for a, b in pairs)
    yield "total_variation", "bounded_counts", b"\2" + b"\xff" * 4096
    for target in ["byte_entropy", "count_entropy", "ngrams", "total_variation"]:
        yield target, "short", b"\xff"
        yield target, "raw_empty", b""


def seeds():
    for target, name, data in base_seeds():
        yield target, name, data
        if target == "total_variation":
            yield "jensen_shannon", name, data
    k = 1 << 50
    for offset in [-2, -1, 0, 1, 2]:
        pairs = [(3*k+offset, k), (k, 3*k+offset)]
        yield "jensen_shannon", f"series_boundary_{offset}", b"\1" + b"".join(
            struct.pack("<QQ", a, b) for a, b in pairs)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    expected = {ROOT / target / (name + ".bin"): data for target, name, data in seeds()}
    for path, data in expected.items():
        if args.check:
            if not path.is_file() or path.read_bytes() != data:
                raise SystemExit(f"Seed missing or changed: {path}")
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
    # Additional hand-retained regression seeds are allowed and replayed too.
    print(f"{'Verified' if args.check else 'Wrote'} {len(expected)} starting seeds")


if __name__ == "__main__":
    main()
