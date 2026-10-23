#!/usr/bin/env python3
"""Independent Jensen-Shannon references: exact mixtures and 160-digit logs."""

import argparse
from decimal import Decimal, localcontext
from fractions import Fraction
from itertools import zip_longest
from pathlib import Path
from reference_total_variation import cases as count_cases


def cases():
    result = count_cases()
    result['a_vs_ab'] = ([1], [1, 1])
    for bits in [30, 60]:
        k = 1 << (bits - 2)
        for offset in [-2, -1, 0, 1, 2]:
            result[f'series_boundary_{bits}_{offset}'] = ([3*k+offset, k], [k, 3*k+offset])
        n = (1 << bits) - 1
        result[f'majority_products_{bits}'] = ([n-1, 1], [n-2, 2])
        for power in [1, 2, 4, 8, 16]:
            result[f'near_identity_{bits}_{power}'] = ([n//2, n//2+1], [n//2-power, n//2+1+power])
    return result


def divergence(left, right):
    n, m = sum(left), sum(right)
    if not n or not m:
        return None
    def dec(value):
        return Decimal(value.numerator) / Decimal(value.denominator)
    # Each probability and mixture is exact before Decimal conversion. This
    # directly evaluates signed KL terms, independently of the Rust gap series.
    with localcontext() as context:
        context.prec = 160
        result = Decimal(0)
        for a, b in zip_longest(left, right, fillvalue=0):
            p, q = Fraction(a, n), Fraction(b, m)
            mixture = (p + q) / 2
            if p:
                result += dec(p) * dec(p / mixture).ln()
            if q:
                result += dec(q) * dec(q / mixture).ln()
        return result / (2 * Decimal(2).ln())


def fixtures():
    rows = ['# case\tleft_counts\tright_counts\tdivergence_bits\tdistance_sqrt_bits']
    with localcontext() as context:
        context.prec = 160
        for name, (left, right) in cases().items():
            j = divergence(left, right)
            if j is None:
                values = 'error\t-'
            else:
                assert j >= 0 and j <= 1 + Decimal('1e-150'), name
                values = f'{j:.80e}\t{j.sqrt():.80e}'
            rows.append(f"{name}\t{','.join(map(str, left)) or '-'}\t"
                        f"{','.join(map(str, right)) or '-'}\t{values}")
    return '\n'.join(rows) + '\n'


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    path = Path(__file__).resolve().parents[1] / 'tests/data/jensen_shannon.tsv'
    expected = fixtures()
    if args.check:
        if path.read_text() != expected:
            raise SystemExit('Jensen-Shannon fixtures differ; regenerate and review.')
        print(f'{len(cases())} independent high-precision Jensen-Shannon fixtures verified.')
    else:
        path.write_text(expected)
        print(f'Wrote {len(cases())} fixtures to {path}')
