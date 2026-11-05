#!/usr/bin/env python3
"""Independent 100-digit mutual information from three empirical entropies."""
import argparse
from collections import Counter
from decimal import Decimal, localcontext
from itertools import product
from pathlib import Path
import random
ROOT=Path(__file__).resolve().parents[1]
def fixtures():
    tables=[]
    for n in range(6):
        for pairs in product([(0,0),(0,255),(255,0),(255,255)],repeat=n):
            tables.append(Counter(pairs))
    rng=random.Random(20261105)
    for k in [2,16,256]:
        for n in [7,16,63,256]:tables.append(Counter((rng.randrange(k),rng.randrange(k)) for _ in range(n)))
    for k in [1,2,16,256,65536]:
        tables.append(Counter({(0,0):k+1,(0,1):k-1,(1,0):k-1,(1,1):k+1}))
    # Ratios d=+/-0.25 straddle the series/direct boundary with fixed margins.
    for offset in [-1,0,1]:
        k=4096;t=k//4+offset
        tables.append(Counter({(0,0):k+t,(0,1):k-t,(1,0):k-t,(1,1):k+t}))
    tables += [Counter({(0,0):6,(0,1):3,(1,0):2,(1,1):1}),Counter({(0,0):999,(1,1):1}),Counter({(0,0):2,(0,1):1,(1,0):1}),Counter({(0,0):1,(1,1):1})]
    rows=['# pair_counts_a:b:c\tmutual_information_bits\tstrict_relative']
    with localcontext() as context:
        context.prec=100;ln2=Decimal(2).ln()
        def entropy(counts):
            n=Decimal(sum(counts))
            if not n:return Decimal(0)
            return n.ln()/ln2-sum(Decimal(c)*Decimal(c).ln()/ln2 for c in counts if c)/n
        for table in tables:
            table=+table
            mx=Counter();my=Counter()
            for (a,b),c in table.items():mx[a]+=c;my[b]+=c
            value=entropy(list(mx.values()))+entropy(list(my.values()))-entropy(list(table.values()))
            # Exact independence is decided in integers, not by Decimal roundoff.
            n=sum(table.values())
            independent=all(table[(a,b)]*n==cx*cy for a,cx in mx.items() for b,cy in my.items())
            if independent:value=Decimal(0)
            assert value>=0
            counts=','.join(f'{a}:{b}:{c}' for (a,b),c in sorted(table.items())) or '-'
            rows.append(f'{counts}\t{float(value):.17g}\t{int(0<value<Decimal("1e-6"))}')
    return '\n'.join(rows)+'\n'
def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--check',action='store_true');args=p.parse_args()
    text=fixtures();path=ROOT/'tests/data/mutual.tsv'
    if args.check:assert path.read_text()==text,'Regenerate mutual information fixtures'
    else:path.write_text(text)
    print(f'{"Verified" if args.check else "Wrote"} {len(text.splitlines())-1} mutual information fixtures')
if __name__=='__main__':main()
