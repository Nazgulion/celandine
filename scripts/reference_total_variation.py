#!/usr/bin/env python3
"""Independent total variation fixtures using exact rational half-L1 distance."""

import argparse
from fractions import Fraction
from itertools import zip_longest
from pathlib import Path
import random


def distance(left, right):
    n, m = sum(left), sum(right)
    if not n or not m:
        return None
    return sum((abs(Fraction(a, n) - Fraction(b, m))
                for a, b in zip_longest(left, right, fillvalue=0)), Fraction()) / 2


def cases():
    result = {
        "both_empty": ([], []), "left_empty": ([], [1]), "right_empty": ([1], []),
        "equal": ([3, 1], [3, 1]), "proportional": ([3, 1], [9, 3]),
        "worked": ([3, 1], [1, 1]), "disjoint": ([3, 0], [0, 7]),
        "binary": ([1, 0, 2], [1, 0, 1]), "different_support": ([1, 1], [1, 1, 1]),
        "full_uniform": ([1] * 256, [3] * 256),
    }
    for bits in [32, 64]:
        n = (1 << bits) - 1
        result[f"disjoint{bits}"] = ([n, 0], [0, n])
        result[f"rare{bits}"] = ([n - 1, 1], [n, 0])
        result[f"unit_cross_difference{bits}"] = ([n - 1, 1], [n - 2, 1])
        result[f"near_uniform{bits}"] = ([n // 2, n // 2 + 1], [n // 2 + 1, n // 2])
        result[f"full{bits}"] = ([n - 255] + [1] * 255, [1] * 255 + [n - 255])
    rng = random.Random(20260920)
    for i in range(128):
        bits = 16 if i < 64 else 64
        limit = (1 << bits) - 1
        width = rng.randrange(1, 257)
        pair = [[rng.getrandbits(bits) // 512 for _ in range(width)] for _ in range(2)]
        for counts in pair:
            counts[0] += 1
            if i % 3 == 0:
                counts[0] = limit - sum(counts[1:])
        result[f"varied_{i}"] = pair
    return result


def fixtures():
    rows = ["# case\tleft_counts\tright_counts\texact_numerator\texact_denominator"]
    for name, (left, right) in cases().items():
        value = distance(left, right)
        if value is not None:
            assert 0 <= value <= 1
            # A separate event-enumeration definition cross-checks small alphabets.
            width = max(len(left), len(right))
            if width <= 8:
                event_gap = max(abs(
                    Fraction(sum(c for i, c in enumerate(left) if mask & (1 << i)), sum(left)) -
                    Fraction(sum(c for i, c in enumerate(right) if mask & (1 << i)), sum(right)))
                    for mask in range(1 << width))
                assert event_gap == value
        expected = "error\t-" if value is None else f"{value.numerator}\t{value.denominator}"
        rows.append(f"{name}\t{','.join(map(str, left)) or '-'}\t"
                    f"{','.join(map(str, right)) or '-'}\t{expected}")
    return "\n".join(rows) + "\n"


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    path = Path(__file__).resolve().parents[1] / "tests/data/total_variation.tsv"
    expected = fixtures()
    if args.check:
        if path.read_text() != expected:
            raise SystemExit("Total variation fixtures differ; regenerate and review.")
        print(f"{len(cases())} exact rational total variation fixtures verified.")
    else:
        path.write_text(expected)
        print(f"Wrote {path}")
