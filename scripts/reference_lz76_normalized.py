#!/usr/bin/env python3
"""Independent substring-novelty counts and 100-digit LZ76 rescaling."""
import argparse
from decimal import Decimal, localcontext
from itertools import product
from pathlib import Path
from reference_lz76 import phrases


def fixtures():
    samples = [bytes(bits) for n in range(10) for bits in product((0, 255), repeat=n)]
    samples += [b"AAAA", b"ABAB", b"ABC", b"101101011", b"010011101101100",
                bytes(range(256)), bytes(range(256))*2, b"A"*1024, "éé".encode()]
    rows = ["# hex\tK\tcount\tsupport\tdensity\tasymptotic_or_error"]
    with localcontext() as ctx:
        ctx.prec = 100
        for data in samples:
            n, c, support = len(data), len(phrases(data)), len(set(data))
            density = Decimal(c)/n if n else Decimal(0)
            for k in sorted({0, 1, 2, 3, max(2, support), 16, 256, 257}):
                if not 2 <= k <= 256:
                    result = "InvalidAlphabetSize"
                elif support > k:
                    result = "AlphabetTooSmall"
                else:
                    result = str(density*Decimal(n).ln()/Decimal(k).ln() if n > 1 else Decimal(0))
                rows.append(f"{data.hex() or '-'}\t{k}\t{c}\t{support}\t{density}\t{result}")
    return "\n".join(rows)+"\n", len(rows)-1


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    path = Path(__file__).resolve().parents[1]/'tests/data/lz76_normalized.tsv'
    content, count = fixtures()
    if args.check:
        if path.read_text() != content: raise SystemExit('Normalized LZ76 fixtures differ; regenerate and review.')
        print(f'Verified {count} independent normalized LZ76 cases.')
    else:
        path.write_text(content)
        print(f'Wrote {count} normalized LZ76 cases.')
