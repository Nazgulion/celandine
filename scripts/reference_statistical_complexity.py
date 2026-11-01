#!/usr/bin/env python3
"""Independent 160-digit entropy-mixture evaluation on an explicit alphabet."""
import argparse
from decimal import Decimal, localcontext
from pathlib import Path
import random
from functools import cache


def cases():
    result = {'worked_binary': ([3, 1], 2), 'worked_four': ([1, 1], 4)}
    for k in range(2, 257):
        result[f'point_{k}'] = ([1], k)
        result[f'uniform_{k}'] = ([1]*k, k)
        result[f'partial_{k}'] = ([1]*(k-1), k)
    for k in [2, 3, 16, 255, 256]:
        for bits in [32, 53, 60, 64]:
            n = 2**bits - 1
            result[f'rare_{k}_{bits}'] = ([n-1, 1], k)
            q = n//k
            counts = [q]*k
            counts[0] -= 1
            counts[-1] += 1
            result[f'near_uniform_{k}_{bits}'] = (counts, k)
    rng = random.Random(20261101)
    for i in range(40):
        k = rng.randrange(2, 257)
        result[f'mixed_{i}'] = ([rng.randrange(0, 100000) for _ in range(k)], k)
    result.update({'empty': ([], 2), 'zero_k': ([1], 0), 'singleton': ([1], 1),
                   'large_k': ([1], 257), 'too_small': ([1, 1, 1], 2), 'invalid_empty': ([], 0)})
    return result


def value(counts, k):
    if not 2 <= k <= 256: return 'alphabet_error'
    if not sum(counts): return 'empty_error'
    counts = [c for c in counts if c]
    if len(counts) > k: return 'support_error'
    with localcontext() as ctx:
        ctx.prec = 160
        D = Decimal
        @cache
        def term(x):
            return x*x.ln()
        def entropy(p):
            return -sum((term(x) for x in p if x), D(0))/D(2).ln()
        p = [D(c)/D(sum(counts)) for c in counts] + [D(0)]*(k-len(counts))
        u = [D(1)/D(k)]*k
        h = entropy(p)
        j = entropy([(a+b)/2 for a,b in zip(p,u)])-(h+entropy(u))/2
        # Maximum is evaluated as a separate mixture, not the Rust closed form.
        delta = [D(1)] + [D(0)]*(k-1)
        maximum = entropy([(a+b)/2 for a,b in zip(delta,u)])-entropy(u)/2
        if len(counts) == 1 or (len(counts)==k and len(set(counts))==1): return D(0)
        c = h/entropy(u)*j/maximum
        assert 0 < c < 1
        return c


def fixtures():
    rows = ['# case\tcounts\talphabet_size\tcomplexity_or_error']
    for name,(counts,k) in cases().items():
        c = value(counts,k)
        formatted = c if isinstance(c,str) else f'{c:.80e}'
        rows.append(f'{name}\t{",".join(map(str,counts)) or "-"}\t{k}\t{formatted}')
    return '\n'.join(rows)+'\n'


if __name__ == '__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check',action='store_true')
    args=parser.parse_args()
    path=Path(__file__).resolve().parents[1]/'tests/data/statistical_complexity.tsv'
    expected=fixtures()
    if args.check: assert path.read_text()==expected, 'Regenerate statistical-complexity fixtures'
    else: path.write_text(expected)
    print(f'{"Verified" if args.check else "Wrote"} {len(cases())} independent statistical-complexity fixtures')
