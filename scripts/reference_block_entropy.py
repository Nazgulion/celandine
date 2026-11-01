#!/usr/bin/env python3
"""Independent sorted-block/run-length oracle with 100-digit Decimal entropy."""
import argparse
from decimal import Decimal, localcontext
from itertools import groupby, product
from pathlib import Path
import random

ROOT = Path(__file__).resolve().parents[1]


def fixtures():
    samples = [bytes(bits) for n in range(7) for bits in product([0, 255], repeat=n)]
    samples += [b'ABABA', b'AABBA', b'AAAB', bytes(range(256)), b'A' * 1023 + b'B']
    rng = random.Random(20261101)
    samples += [bytes(rng.randrange(k) for _ in range(64)) for k in [2, 4, 16, 256]]
    rows = ['# input_hex\tn\tentropy_bits_per_block']
    with localcontext() as ctx:
        ctx.prec = 100
        log2 = Decimal(2).ln()
        for data in samples:
            lengths = list(range(min(len(data) + 3, 10))) + [len(data), len(data) + 1, 'max']
            for n in dict.fromkeys(lengths):
                width = 2**64 - 1 if n == 'max' else n
                if width == 0:
                    result = 'error'
                else:
                    blocks = sorted(data[i:i+width] for i in range(max(0, len(data)-width+1)))
                    counts = [Decimal(sum(1 for _ in group)) for _, group in groupby(blocks)]
                    total = sum(counts, Decimal(0))
                    # H = log2(m) - sum(c*log2(c))/m, independent evaluation form.
                    h = (total.ln() - sum(c*c.ln() for c in counts)/total)/log2 if total else Decimal(0)
                    result = format(float(h), '.17g')
                rows.append(f'{data.hex() or "-"}\t{n}\t{result}')
    return '\n'.join(rows) + '\n'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    content = fixtures()
    path = ROOT / 'tests/data/block_entropy.tsv'
    if args.check:
        assert path.read_text() == content, 'Regenerate block entropy fixtures'
    else:
        path.write_text(content)
    print(f'{"Verified" if args.check else "Wrote"} {len(content.splitlines())-1} block entropy fixtures')


if __name__ == '__main__':
    main()
