#!/usr/bin/env python3
"""Independent Tsallis fixtures: direct powers with 120-digit Decimal arithmetic."""

import argparse
from decimal import Decimal, localcontext
from pathlib import Path

ORDERS = [
    "0", "-0", "5e-324", "2.2250738585072014e-308", "0.1", "0.5", "0.9",
    "0.99999999", "0.999999999999", "0.9999999999999999", "1",
    "1.0000000000000002", "1.000000000001", "1.00000001", "1.1", "2",
    "4", "16", "128", "1000000", "1e18", "1e20", "1e100",
    "1.7976931348623157e308",
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
        ln_two = Decimal(2).ln()
        total = sum(counts)
        probabilities = [Decimal(c) / Decimal(total) for c in counts]
        q = Decimal.from_float(float(order))
        if q == 0:
            return Decimal(len(counts)-1) / ln_two
        if q == 1:
            return -sum(p * p.ln() for p in probabilities) / ln_two
        if q >= Decimal('1e100'):
            # Nonconstant counts, n<=2^64-1: p_max<=1-1/n.
            # Relative error from dropping the power sum is at most
            # 256*exp(-q/(2^64-1)) / (1-256*exp(-q/(2^64-1))).
            # This is far below 1e-120 for these orders. Keep scientific output
            # so positive subnormal binary64 references do not become zero.
            return 1 / (q-1) / ln_two
        return (1-sum(p**q for p in probabilities)) / (q-1) / ln_two


def varied_cases() -> dict[str, list[int]]:
    state = 0x5453414C
    mask = 2**64-1
    def next_value() -> int:
        nonlocal state
        state ^= (state << 13) & mask
        state ^= state >> 7
        state ^= (state << 17) & mask
        return state
    result = {}
    for i in range(16):
        counts = [1+(next_value() >> (next_value()%56))//256 for _ in range(2+next_value()%31)]
        if i%2 == 0:
            counts[0] = mask-sum(counts[1:])
        assert sum(counts) <= mask
        result[f"varied64_{i}"] = counts
    return result


def near_zero_cases() -> dict[str, list[int]]:
    # Review regressions: repeated rare probabilities amplify exp/log error when
    # the near-one formula is used near zero. Include the formula boundary too.
    return {
        f"near_zero_full_{shift}": [2**64-1-255*(1 << shift)] + [1 << shift]*255
        for shift in [0, 1, 16, 55]
    }


def fixtures() -> str:
    rows = ["# case\tq_binary64\tcounts\tscaled_tsallis (120 digits; >=1e100 bounded asymptotic)"]
    groups = [
        (CASES, ORDERS),
        (varied_cases(), ["0.1", "0.5", "0.9999999999999999", "1.0000000000000002", "2", "1e20", "1.7976931348623157e308"]),
        (near_zero_cases(), ["5e-324", "1e-18", "1e-17", "3e-17", "5e-17", "6e-17", "1e-16", "3e-16", "1e-12", "1e-8", "0.001", "0.01", "0.49999999999999994", "0.5", "0.5000000000000001"]),
    ]
    for cases, orders in groups:
        for name, counts in cases.items():
            for order in orders:
                h = entropy(counts, order)
                if h == 0:
                    h = abs(h)
                rows.append(f"{name}\t{order}\t{','.join(map(str, counts)) or '-'}\t{h:.60e}")
    return "\n".join(rows)+"\n"


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="verify fixtures without writing")
    args = parser.parse_args()
    path = Path(__file__).resolve().parents[1] / "tests" / "data" / "tsallis.tsv"
    expected = fixtures()
    if args.check:
        if path.read_text() != expected:
            raise SystemExit("Tsallis reference fixtures differ; regenerate and review them.")
        print(f"{len(expected.splitlines())-1} independent high-precision Tsallis fixtures verified.")
    else:
        path.write_text(expected)
        print(f"Wrote {path}")
