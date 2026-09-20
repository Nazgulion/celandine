#!/usr/bin/env python3
"""Independent exact-count, 120-digit Shannon fixtures for explicit f64 bases."""

import argparse
from decimal import Decimal, localcontext
from fractions import Fraction
import math
from pathlib import Path
import struct


def fixtures() -> str:
    cases = [
        ("empty", []), ("singleton", [1]), ("constant", [42]),
        ("two", [1, 1]), ("four", [1] * 4), ("thirds", [1] * 3),
        ("skewed", [3, 1]), ("full_alphabet", [1] * 256),
        ("rare", [4095, 1]), ("balanced_wide", [2**63, 2**63 - 1]),
        ("rare_wide", [2**64 - 2, 1]), ("constant_wide", [2**64 - 1]),
    ]
    bases = [math.nextafter(1.0, math.inf), 1 + 2**-40, 1.0001, 1.5,
             math.nextafter(2.0, 0), 2.0, math.nextafter(2.0, math.inf),
             math.e, 10.0, 256.0, 1e100, float.fromhex('0x1.fffffffffffffp+1023')]
    rows = ["# name\tcounts\tbase_f64_bits_hex\tentropy (120-digit direct natural-log calculation)"]
    with localcontext() as ctx:
        ctx.prec = 120
        for name, counts in cases:
            total = sum(counts)
            for base in bases:
                log_base = Decimal.from_float(base).ln()
                h = Decimal(0)
                for count in counts:
                    rational = Fraction(count, total)
                    p = Decimal(rational.numerator) / Decimal(rational.denominator)
                    h -= p * p.ln() / log_base
                bits = struct.unpack('>Q', struct.pack('>d', base))[0]
                count_text = ','.join(map(str, counts)) or '-'
                rows.append(f'{name}\t{count_text}\t{bits:016x}\t{h:.80e}')
    return '\n'.join(rows) + '\n'


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true', help='verify fixtures without writing')
    args = parser.parse_args()
    path = Path(__file__).resolve().parents[1] / 'tests/data/shannon_base.tsv'
    expected = fixtures()
    if args.check:
        if path.read_text() != expected:
            raise SystemExit('Shannon-base fixtures differ; regenerate and review them.')
        print('144 independent high-precision Shannon-base fixtures verified.')
    else:
        path.write_text(expected)
        print(f'Wrote {path}')
