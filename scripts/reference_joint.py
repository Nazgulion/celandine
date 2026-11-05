#!/usr/bin/env python3
"""Independent aligned-pair counts and 100-digit joint entropy fixtures."""
import argparse
from collections import Counter
from decimal import Decimal, localcontext
from functools import lru_cache
from itertools import product
from pathlib import Path
import random
ROOT=Path(__file__).resolve().parents[1]
def fixtures():
    samples=[]
    for n in range(6):
        for pairs in product([(0,0),(0,255),(255,0),(255,255)],repeat=n):
            samples.append((bytes(a for a,b in pairs),bytes(b for a,b in pairs)))
    samples += [(b'AABB',b'XYXY'),(b'AABB',b'XXYY'),(b'AAAB',b'XXYX'),(bytes(range(256)),bytes(reversed(range(256))))]
    rng=random.Random(20261105)
    for k in [2,16,256]:
        for n in [7,16,63,256]:
            samples.append((bytes(rng.randrange(k) for _ in range(n)),bytes(rng.randrange(k) for _ in range(n))))
    samples += [(bytes(a),bytes(b)) for a,b in [(0,1),(1,0),(1,2),(2,1),(16,17),(17,16)]]
    rows=['# x_hex\ty_hex\tpair_counts_a:b:c\tjoint_entropy_bits']
    with localcontext() as context:
        context.prec=100
        ln2=Decimal(2).ln()
        @lru_cache(None)
        def entropy(counts):
            if not counts:return 0.0
            n=Decimal(sum(counts))
            return float((n.ln()-sum(Decimal(c)*Decimal(c).ln() for c in counts)/n)/ln2)
        for x,y in samples:
            if len(x)!=len(y): counts='error'; h='error'
            else:
                c=Counter(zip(x,y))
                counts=','.join(f'{a}:{b}:{n}' for (a,b),n in sorted(c.items())) or '-'
                h=format(entropy(tuple(sorted(c.values()))),'.17g')
            rows.append(f'{x.hex() or "-"}\t{y.hex() or "-"}\t{counts}\t{h}')
    return '\n'.join(rows)+'\n'
def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--check',action='store_true');args=parser.parse_args()
    result=fixtures();path=ROOT/'tests/data/joint.tsv'
    if args.check:assert path.read_text()==result,'Regenerate joint fixtures'
    else:path.write_text(result)
    print(f'{"Verified" if args.check else "Wrote"} {len(result.splitlines())-1} joint fixtures')
if __name__=='__main__':main()
