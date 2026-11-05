#!/usr/bin/env python3
"""Independent matched-sample chain rule using 100-digit Decimal arithmetic."""
import argparse
from collections import Counter
from decimal import Decimal, localcontext
from itertools import product
from pathlib import Path
import random

ROOT = Path(__file__).resolve().parents[1]


def fixtures():
    samples = [bytes(bits) for n in range(8) for bits in product([0, 255], repeat=n)]
    samples += [b'ABABAB', b'AAB', b'AABA', b'AAAABBA', bytes(range(256)), b'A'*1023+b'BA']
    rng = random.Random(20261105)
    samples += [bytes(rng.randrange(k) for _ in range(64)) for k in [2, 4, 16, 256]]
    rows = ['# input_hex\torder\tconditional_bits_per_predicted_byte']
    with localcontext() as ctx:
        ctx.prec = 100
        ln2 = Decimal(2).ln()
        def entropy(counts):
            total = Decimal(sum(counts.values()))
            return (total.ln()-sum(Decimal(c)*Decimal(c).ln() for c in counts.values())/total)/ln2 if total else Decimal(0)
        for data in samples:
            orders = list(range(min(len(data)+2, 10))) + [len(data)-1 if data else 0, len(data), len(data)+1, 'max']
            for order in dict.fromkeys(orders):
                k = 2**64-1 if order == 'max' else order
                pairs, contexts = Counter(), Counter()
                for t in range(k, len(data)):
                    context = tuple(data[t-k:t])
                    pairs[(context, data[t])] += 1
                    contexts[context] += 1
                result = entropy(pairs)-entropy(contexts)
                rows.append(f'{data.hex() or "-"}\t{order}\t{float(result):.17g}')
    return '\n'.join(rows)+'\n'


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check',action='store_true')
    args=parser.parse_args()
    content=fixtures()
    path=ROOT/'tests/data/conditional_order.tsv'
    if args.check:
        assert path.read_text()==content, 'Regenerate conditional entropy fixtures'
    else:
        path.write_text(content)
    print(f'{"Verified" if args.check else "Wrote"} {len(content.splitlines())-1} conditional entropy fixtures')


if __name__=='__main__': main()
