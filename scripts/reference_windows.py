#!/usr/bin/env python3
"""Enumerate valid start positions independently of the Rust iterator adapters."""
import argparse
from pathlib import Path
ROOT = Path(__file__).resolve().parents[1]


def fixtures():
    rows = ['# input_hex\tsize\tstep\tstart_offsets_or_error']
    samples = [bytes(range(n)) for n in range(9)] + [b'ABABA', b'\0\xff\0\xff', 'éé'.encode()]
    parameters = list(range(11)) + ['max']
    for data in samples:
        for width in parameters:
            for stride in parameters:
                w = 2**64-1 if width == 'max' else width
                h = 2**64-1 if stride == 'max' else stride
                if not w: result = 'size_error'
                elif not h: result = 'step_error'
                else:
                    positions = [i for i in range(len(data)) if i % h == 0 and i+w <= len(data)]
                    result = ','.join(map(str,positions)) or '-'
                rows.append(f'{data.hex() or "-"}\t{width}\t{stride}\t{result}')
    return '\n'.join(rows)+'\n'


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check',action='store_true')
    args=parser.parse_args()
    data=fixtures(); path=ROOT/'tests/data/windows.tsv'
    if args.check:
        assert path.read_text()==data, 'Regenerate window fixtures'
    else: path.write_text(data)
    print(f'{"Verified" if args.check else "Wrote"} {len(data.splitlines())-1} independent window cases')


if __name__=='__main__': main()
