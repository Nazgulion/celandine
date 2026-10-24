#!/usr/bin/env python3
"""Reference partitioning by accumulating bytes until a block is complete."""
import argparse
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def fixtures():
    rows = ['# input_hex\tsize\tblocks_hex_or_error\tremainder_hex']
    samples = [bytes(range(n)) for n in range(21)] + [b'ABABA', b'AAAABABAZ', b'\0\xff\0\xff', 'éé'.encode()]
    for data in samples:
        for size in list(range(12)) + ['max']:
            width = 2**64 - 1 if size == 'max' else size
            if width == 0:
                result, tail = 'error', '-'
            else:
                complete, partial = [], bytearray()
                for byte in data:
                    partial.append(byte)
                    if len(partial) == width:
                        complete.append(partial.hex())
                        partial.clear()
                result = ','.join(complete) or '-'
                tail = partial.hex() or '-'
            rows.append(f'{data.hex() or "-"}\t{size}\t{result}\t{tail}')
    return '\n'.join(rows) + '\n'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true')
    args = parser.parse_args()
    data = fixtures()
    path = ROOT / 'tests/data/blocks.tsv'
    if args.check:
        assert path.read_text() == data, 'Regenerate block fixtures'
    else:
        path.write_text(data)
    print(f'{"Verified" if args.check else "Wrote"} {len(data.splitlines()) - 1} independent block cases')


if __name__ == '__main__':
    main()
