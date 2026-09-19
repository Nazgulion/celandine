#!/usr/bin/env python3
"""Independent Rényi fixtures: direct definition in 120-digit Decimal arithmetic."""

import argparse
import math
from decimal import Decimal, localcontext
from pathlib import Path

ORDERS = [
    "0", "-0", "5e-324", "0.1", "0.5", "0.7499999999999999", "0.75",
    "0.99999999", "0.999999999999", "0.9999999999999999", "1",
    "1.0000000000000002", "1.000000000001", "1.00000001", "1.25",
    "1.2500000000000002", "2", "16", "128", "1000000", "1e100",
    "1.7976931348623157e308", "inf",
]
CASES = {
    "empty": [],
    "constant": [7],
    "binary_uniform": [3, 3],
    "ternary_uniform": [7, 7, 7],
    "full_uniform": [1] * 256,
    "aaab": [3, 1],
    "uneven": [1, 2, 3, 5, 8, 13, 21],
    "geometric": [2**i for i in range(20)],
    "rare": [1000000, 1, 1, 1],
    "near_uniform": [1000001, 1000000, 999999],
    "tied_maxima": [1000000, 1000000, 1],
    "usize32_rare": [2**32 - 2, 1],
    "usize64_rare": [2**64 - 2, 1],
    "usize64_full": [2**64 - 256] + [1] * 255,
    "usize64_tied": [2**63 - 1, 2**63 - 1, 1],
}


def entropy(counts: list[int], order: str) -> Decimal:
    counts = [c for c in counts if c]
    if len(counts) <= 1:
        return Decimal(0)
    with localcontext() as ctx:
        ctx.prec = 120
        ctx.Emax = 10**12
        ctx.Emin = -(10**12)
        two_log = Decimal(2).ln()
        probabilities = [Decimal(c) / Decimal(sum(counts)) for c in counts]
        # The reference uses the exact binary64 order passed to Rust.
        alpha_float = float(order)
        if alpha_float == 0:
            return Decimal(len(counts)).ln() / two_log
        if alpha_float == 1:
            return -sum(p * p.ln() for p in probabilities) / two_log
        h_inf = -max(probabilities).ln() / two_log
        if math.isinf(alpha_float) or alpha_float >= 1e100:
            # For finite a > 1, 0 <= H_a-H_inf <= log2(k)/(a-1).
            # Here that is < 9e-100 bits, far below the 60-place output.
            return h_inf
        alpha = Decimal.from_float(alpha_float)
        # Direct high-precision powers and logarithm, independent of Rust's
        # near-one expm1 and maximum-scaled evaluation branches.
        return sum(p**alpha for p in probabilities).ln() / (1 - alpha) / two_log


def fixtures() -> str:
    rows = ["# case\talpha_binary64\tcounts\tentropy_bits (120-digit reference; 60 places)"]
    for name, counts in CASES.items():
        for order in ORDERS:
            rows.append(f"{name}\t{order}\t{','.join(map(str, counts)) or '-'}\t{entropy(counts, order):.60f}")
    return "\n".join(rows) + "\n"


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="verify fixtures without writing")
    args = parser.parse_args()
    path = Path(__file__).resolve().parents[1] / "tests" / "data" / "renyi.tsv"
    expected = fixtures()
    if args.check:
        if path.read_text() != expected:
            raise SystemExit("Rényi reference fixtures differ; regenerate and review them.")
        print(f"{len(CASES) * len(ORDERS)} independent high-precision Rényi fixtures verified.")
    else:
        path.write_text(expected)
        print(f"Wrote {path}")
