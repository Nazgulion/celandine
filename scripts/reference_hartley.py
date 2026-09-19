#!/usr/bin/env python3
"""Independent Hartley reference for every byte-support size; standard library only."""

import argparse
from decimal import Decimal, localcontext
from pathlib import Path


def entropy(data: bytes) -> Decimal:
    support = len(set(data))
    if support == 0:
        return Decimal(0)  # Explicit project convention, not log(0).
    with localcontext() as context:
        context.prec = 80
        return Decimal(support).ln() / Decimal(2).ln()


def fixtures() -> str:
    rows = ["# support_size\thartley_bits (80-digit Decimal calculation)"]
    for k in range(257):
        # Count a set independently of Rust's fixed histogram, with unequal counts.
        data = b"".join(bytes([i]) * (i % 7 + 1) for i in reversed(range(k)))
        rows.append(f"{k}\t{entropy(data):.60f}")
    return "\n".join(rows) + "\n"


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="verify committed fixtures without writing")
    args = parser.parse_args()
    path = Path(__file__).resolve().parents[1] / "tests" / "data" / "hartley.tsv"
    expected = fixtures()
    if args.check:
        if path.read_text() != expected:
            raise SystemExit("Hartley reference fixtures differ; regenerate and review them.")
        print("257 independent high-precision Hartley support fixtures verified.")
    else:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(expected)
        print(f"Wrote {path}")
