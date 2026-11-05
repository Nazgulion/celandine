#!/usr/bin/env python3
"""Independent 100-digit overlapping-block profiles with explicit output caps."""
import argparse
from collections import Counter
from decimal import Decimal, localcontext
from functools import lru_cache
from itertools import product
from pathlib import Path
import random

ROOT=Path(__file__).resolve().parents[1]


def fixtures():
    samples=[bytes(bits) for n in range(8) for bits in product([0,255],repeat=n)]
    samples += [b'AABB',b'ABABA',b'ABCD',b'AAABAA',bytes(range(256))]
    rng=random.Random(20261105)
    samples += [bytes(rng.randrange(k) for _ in range(32)) for k in [2,4,16,256]]
    rows=['# input_hex\tmax_order\tentropies_bits_per_block']
    with localcontext() as ctx:
        ctx.prec=100
        ln2=Decimal(2).ln()
        @lru_cache(None)
        def entropy(counts):
            total=Decimal(sum(counts))
            return float((total.ln()-sum(Decimal(c)*Decimal(c).ln() for c in counts)/total)/ln2)
        for data in samples:
            profile=[]
            for n in range(1,len(data)+1):
                counts=Counter(data[i:i+n] for i in range(len(data)-n+1))
                profile.append(entropy(tuple(sorted(counts.values()))))
            for maximum in dict.fromkeys([0,1,2,3,len(data)//2,len(data),len(data)+1,'max']):
                limit=len(data) if maximum=='max' else min(maximum,len(data))
                result=','.join(format(h,'.17g') for h in profile[:limit]) or '-'
                rows.append(f'{data.hex() or "-"}\t{maximum}\t{result}')
    return '\n'.join(rows)+'\n'


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check',action='store_true')
    args=parser.parse_args()
    content=fixtures();path=ROOT/'tests/data/block_profile.tsv'
    if args.check: assert path.read_text()==content,'Regenerate block profile fixtures'
    else: path.write_text(content)
    print(f'{"Verified" if args.check else "Wrote"} {len(content.splitlines())-1} block profile fixtures')


if __name__=='__main__': main()
