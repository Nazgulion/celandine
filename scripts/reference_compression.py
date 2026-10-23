#!/usr/bin/env python3
"""Verify actual counted miniz streams with Python's independent C-zlib decoder."""
import argparse
import subprocess
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--check', action='store_true', help='verify live streams (also the default)')
    parser.parse_args()
    command = ['cargo', 'test', '--locked', '--features', 'compression-deflate', '--lib',
               'compression::deflate::tests::export_reference_streams', '--', '--ignored', '--nocapture']
    output = subprocess.check_output(command, cwd=ROOT, text=True)
    count = 0
    for line in output.splitlines():
        if not line.startswith('CELANDINE_COMPRESSION_REFERENCE\t'):
            continue
        _, raw, encoded, length, ratio = line.split('\t')
        data, stream = bytes.fromhex(raw), bytes.fromhex(encoded)
        decoder = zlib.decompressobj()
        decoded = decoder.decompress(stream) + decoder.flush()
        assert decoded == data
        assert decoder.eof and not decoder.unused_data and not decoder.unconsumed_tail
        assert len(stream) == int(length)
        assert stream[:2] == bytes.fromhex('789c')  # DEFLATE, 32 KiB, no dictionary, level hint.
        assert int.from_bytes(stream[-4:], 'big') == zlib.adler32(data)
        if data:
            assert float(ratio) == len(stream) / len(data)
        else:
            assert ratio == 'empty'
        if data in [b'', b'A']:
            # Fixed Huffman end marker; A's literal; independent hand-parsed RFC fields.
            expected = '789c030000000001' if not data else '789c73040000420042'
            assert stream.hex() == expected
        count += 1
    assert count == 27, count
    print(f'Verified {count} counted streams, framing, checksums and ratios with C-zlib {zlib.ZLIB_RUNTIME_VERSION}')


if __name__ == '__main__':
    main()
