#!/usr/bin/env python3
"""Exact LZ76 exhaustive histories by substring novelty, independent of source scanning."""
import argparse
from itertools import product
from pathlib import Path


def phrases(data):
    start = 0
    result = []
    while start < len(data):
        for end in range(start + 1, len(data) + 1):
            # A whole candidate occurs earlier iff found before its last byte.
            # This admits overlapping copies and all earlier starting positions.
            if data[start:end] not in data[:end-1] or end == len(data):
                result.append(data[start:end])
                start = end
                break
    return result


def fixtures():
    samples = [bytes(bits) for size in range(10) for bits in product((0, 1), repeat=size)]
    samples += [b"A", b"AAAA", b"ABABAB", b"ABCD", b"ABCABCABC", b"101101011",
                b"010011101101100", bytes(range(256)), bytes(range(256))*2,
                "éé".encode(), bytes([255,0,128,255,0]), b"A"*1024]
    # Independently hand/paper-derived parsing anchors, including a repeated tail.
    assert phrases(b"AAAA") == [b"A", b"AAA"]
    assert phrases(b"101101011") == [b"1",b"0",b"11",b"010",b"11"]
    assert phrases(b"010011101101100") == [b"0",b"1",b"00",b"11",b"101",b"101100"]
    rows = ["# input_hex\tphrase_hex_list\tcount"]
    for data in samples:
        parts = phrases(data)
        rows.append(f"{data.hex() or '-'}\t{','.join(p.hex() for p in parts) or '-'}\t{len(parts)}")
    return "\n".join(rows)+"\n",len(samples)


if __name__ == "__main__":
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check",action="store_true")
    args=parser.parse_args()
    path=Path(__file__).resolve().parents[1]/"tests/data/lz76.tsv"
    expected,count=fixtures()
    if args.check:
        if path.read_text()!=expected: raise SystemExit("LZ76 fixtures differ; regenerate and review.")
        print(f"Verified {count} independent exhaustive LZ76 histories.")
    else:
        path.write_text(expected)
        print(f"Wrote {count} LZ76 histories.")
