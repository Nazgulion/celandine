#!/usr/bin/env python3
"""Independent high-precision Shannon fixtures; Python standard library only."""

import argparse
from collections import Counter
from decimal import Decimal, localcontext
from fractions import Fraction
from pathlib import Path


def entropy(data: bytes) -> Decimal:
    if not data:
        return Decimal(0)  # Project convention, not a probability law.
    with localcontext() as context:
        context.prec = 80
        total = Decimal(0)
        for count in Counter(data).values():
            rational = Fraction(count, len(data))
            p = Decimal(rational.numerator) / Decimal(rational.denominator)
            total -= p * p.ln() / Decimal(2).ln()
        return +total


def fixtures() -> str:
    cases = [
        ("empty", b""), ("singleton", b"A"), ("constant", b"AAAA"),
        ("two", b"AB"), ("two_repeated", b"ABAB"), ("four", b"ABCD"),
        ("thirds", b"ABCABCABC"), ("ten", b"0123456789"),
        ("skewed", b"AAAB"), ("binary_values", bytes([0, 0, 128, 255])),
        ("full_alphabet", bytes(range(256))),
        ("rare_symbol", bytes(4095) + bytes([255])),
    ]
    for k in [3, 7, 11, 31, 127, 255]:
        cases.append((f"uniform_{k}", bytes(range(k)) * 3))
        cases.append((f"unequal_{k}", b"".join(bytes([i]) * (i % 13 + 1) for i in range(k))))
    rows = ["# name\tinput_hex\tshannon_bits (80-digit Decimal calculation)"]
    rows.extend(f"{name}\t{data.hex()}\t{entropy(data):.60f}" for name, data in cases)
    return "\n".join(rows) + "\n"


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="verify committed fixtures without writing")
    args = parser.parse_args()
    path = Path(__file__).resolve().parents[1] / "tests" / "data" / "shannon.tsv"
    expected = fixtures()
    if args.check:
        if path.read_text() != expected:
            raise SystemExit("Reference fixtures differ; regenerate and review them.")
        print("24 independent high-precision Shannon fixtures verified.")
    else:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(expected)
        print(f"Wrote {path}")
