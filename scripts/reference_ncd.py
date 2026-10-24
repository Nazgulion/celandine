#!/usr/bin/env python3
"""Exact rational NCD fixtures and independent decoding of measured zlib pairs."""
import argparse
from fractions import Fraction
from itertools import product
from pathlib import Path
import subprocess
import zlib
ROOT = Path(__file__).resolve().parents[1]


def fixtures():
    triples = list(product(range(8), repeat=3))
    for maximum in [2**32-1, 2**53, 2**53+1, 2**63, 2**64-1]:
        for delta in [-1, 0, 1]:
            triples.append((maximum, maximum, min(2**64-1, maximum+delta)))
        triples.extend([(maximum, 1, 0), (0, maximum, maximum), (1, 1, maximum)])
    lines = ['# C(x)\tC(y)\tC(xy)\tNCD (Python Fraction -> nearest binary64)']
    for x, y, xy in triples:
        value = repr(float(Fraction(xy-min(x,y),max(x,y)))) if max(x,y) else 'undefined'
        lines.append(f'{x}\t{y}\t{xy}\t{value}')
    return '\n'.join(lines)+'\n'


def check_streams():
    cmd = ['cargo','test','--locked','--features','compression-deflate','--lib',
           'compression::deflate::tests::export_ncd_streams','--','--ignored','--nocapture']
    output = subprocess.check_output(cmd,cwd=ROOT,text=True)
    count = 0
    for line in output.splitlines():
        if not line.startswith('CELANDINE_NCD_REFERENCE\t'): continue
        _, x, y, cx, cy, cxy, nx, ny, nxy, result = line.split('\t')
        x,y = bytes.fromhex(x),bytes.fromhex(y)
        lengths = []
        for encoded, raw, length in [(cx,x,nx),(cy,y,ny),(cxy,x+y,nxy)]:
            stream = bytes.fromhex(encoded)
            decoder = zlib.decompressobj()
            assert decoder.decompress(stream)+decoder.flush() == raw
            assert decoder.eof and not decoder.unused_data and not decoder.unconsumed_tail
            assert len(stream) == int(length)
            lengths.append(len(stream))
        a,b,c = lengths
        expected = float(Fraction(c-min(a,b),max(a,b)))
        assert abs(float(result)-expected) <= abs(expected)*4*2**-52
        count += 1
    assert count == 14,count
    print(f'Checked {count} NCD pairs: independent C-zlib {zlib.ZLIB_RUNTIME_VERSION} decoding + exact rational arithmetic')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check',action='store_true')
    args = parser.parse_args()
    path = ROOT/'tests/data/ncd.tsv'
    expected = fixtures()
    if args.check:
        assert path.read_text() == expected, 'Regenerate NCD fixtures'
        check_streams()
    else:
        path.write_text(expected)
    print(f'{"Verified" if args.check else "Wrote"} 542 exact-rational NCD triples')


if __name__ == '__main__': main()
