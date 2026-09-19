#!/usr/bin/env python3
"""Independent collision entropy: exact rational matching counts and Decimal logs."""

import argparse
from decimal import Decimal, localcontext
from fractions import Fraction
from pathlib import Path


def entropy(counts: list[int]) -> Decimal:
    total = sum(counts)
    if not total:
        return Decimal(0)  # Project convention; matching probability is undefined.
    # Python integers do not overflow when squaring large imported counts.
    matching = Fraction(sum(c * c for c in counts), total * total)
    with localcontext() as context:
        context.prec = 120
        probability = Decimal(matching.numerator) / Decimal(matching.denominator)
        return -probability.ln() / Decimal(2).ln()


def cases() -> dict[str, list[int]]:
    result = {
        "empty": [], "constant": [4], "ab": [1, 1], "aaab": [3, 1],
        "abc": [3, 3, 3], "digits": [1] * 10, "full_uniform": [1] * 256,
        "uneven": [1, 2, 3, 5, 8, 13, 21],
        "near_uniform": [1000001, 1000000, 999999],
        "usize32_rare": [2**32 - 2, 1],
        "usize64_rare": [2**64 - 2, 1],
        "usize64_full": [2**64 - 256] + [1] * 255,
        "usize64_tied": [2**63 - 1, 2**63 - 1, 1],
    }
    # Fixed xorshift64 generates varied magnitudes independently of Rust tests.
    state = 0xC0111510
    mask = 2**64 - 1

    def next_value() -> int:
        nonlocal state
        state ^= (state << 13) & mask
        state ^= state >> 7
        state ^= (state << 17) & mask
        return state

    for i in range(32):
        counts = []
        for _ in range(2 + next_value() % 31):
            shift = next_value() % 56
            counts.append(1 + (next_value() >> shift) // 256)
        if i % 2 == 0:
            counts[0] = mask - sum(counts[1:])
        assert sum(counts) <= mask
        result[f"varied64_{i}"] = counts
    return result


def fixtures() -> str:
    rows = ["# case\tcounts\tentropy_bits (exact Fraction, 120-digit Decimal, 60 places)"]
    for name, counts in cases().items():
        h = entropy(counts)
        if h == 0:
            h = abs(h)
        rows.append(f"{name}\t{','.join(map(str, counts)) or '-'}\t{h:.60f}")
    return "\n".join(rows) + "\n"


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="verify fixtures without writing")
    args = parser.parse_args()
    path = Path(__file__).resolve().parents[1] / "tests" / "data" / "collision.tsv"
    expected = fixtures()
    if args.check:
        if path.read_text() != expected:
            raise SystemExit("Collision reference fixtures differ; regenerate and review them.")
        print(f"{len(cases())} independent high-precision collision fixtures verified.")
    else:
        path.write_text(expected)
        print(f"Wrote {path}")
