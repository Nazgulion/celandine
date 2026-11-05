#!/usr/bin/env python3
"""Independent 100-digit aligned conditional entropy via H(X,Y)-H(Y)."""
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
    tables += [Counter(zip(x,y)) for x,y in [(b'ABCD',b'XXYY'),(b'XXYY',b'ABCD'),(b'AAAABC',b'XXXXYY'),(b'AAAB',b'XXYY'),(b'XXYY',b'AAAB')]]
    for k in [2,256,65536,262144]:
        tables += [Counter({(0,0):k,(1,0):1}),Counter({(0,0):1,(1,0):1,(2,1):k}),Counter({(0,0):k,(1,1):1})]
    # Conditional-probability branches on either side of one half.
    for offset in [-1,0,1]:tables.append(Counter({(0,0):64+offset,(1,0):64-offset,(2,1):3}))
    rows=['# pair_counts_a:b:c\tH_X_given_Y_bits\tstrict_relative']
    with localcontext() as context:
        context.prec=100;ln2=Decimal(2).ln()
        def entropy(counts):
            n=Decimal(sum(counts))
            if not n:return Decimal(0)
            return n.ln()/ln2-sum(Decimal(c)*Decimal(c).ln()/ln2 for c in counts if c)/n
        for table in tables:
            table=+table;my=Counter()
            for (a,b),c in table.items():my[b]+=c
            value=entropy(list(table.values()))-entropy(list(my.values()))
            deterministic=len(table)==len(my)
            if deterministic:value=Decimal(0)
            assert value>=0
            counts=','.join(f'{a}:{b}:{c}' for (a,b),c in sorted(table.items())) or '-'
            rows.append(f'{counts}\t{float(value):.17g}\t{int(0<value<Decimal("1e-4"))}')
    return '\n'.join(rows)+'\n'
def main():
    p=argparse.ArgumentParser(description=__doc__);p.add_argument('--check',action='store_true');args=p.parse_args()
    text=fixtures();path=ROOT/'tests/data/conditional_pair.tsv'
    if args.check:assert path.read_text()==text,'Regenerate aligned conditional entropy fixtures'
    else:path.write_text(text)
    print(f'{"Verified" if args.check else "Wrote"} {len(text.splitlines())-1} aligned conditional entropy fixtures')
if __name__=='__main__':main()
