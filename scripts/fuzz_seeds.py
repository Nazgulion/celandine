#!/usr/bin/env python3
"""Generate or verify the small, portable libFuzzer starting corpus."""

import argparse
import math
from pathlib import Path
import struct

ROOT = Path(__file__).resolve().parents[1] / "fuzz" / "seeds"


def base_seeds():
    values = [
        ("zero", 0.0), ("negative_zero", -0.0), ("negative", -1.0),
        ("half", 0.5), ("one_below", math.nextafter(1.0, 0.0)),
        ("one", 1.0), ("one_above", math.nextafter(1.0, math.inf)),
        ("two", 2.0), ("ten", 10.0), ("tiny", math.ulp(0.0)),
        ("huge", float.fromhex("0x1.fffffffffffffp+1023")),
        ("infinity", math.inf), ("negative_infinity", -math.inf),
        ("nan", math.nan),
    ]
    maximum = (1 << 64) - 1
    for name, value in values:
        header = struct.pack("<dd", value, value)
        yield "byte_entropy", name, header + b"AAAB"
        yield "count_entropy", name, header + struct.pack("<QQ", maximum - 1, 1)
    header = struct.pack("<dd", 0.5, 2.0)
    for name, data in [
        ("empty", b""), ("constant", b"A" * 64), ("uniform", bytes(range(256))),
        ("maximum_length", bytes(range(256)) * 16), ("binary", b"\x00\xff\x00"),
    ]:
        yield "byte_entropy", name, header + data
    for name, counts in [
        ("empty", []), ("constant", [maximum]), ("balanced", [5] * 256),
        ("overflow_one", [maximum, 1]), ("overflow_many", [maximum] * 256),
        ("exact_maximum", [maximum - 1000, 999, 1]),
    ]:
        yield "count_entropy", name, header + b"".join(struct.pack("<Q", c) for c in counts)
    for name, n, data in [
        ("zero_empty", 0, b""), ("zero_nonempty", 0, b"AB"),
        ("empty", 1, b""), ("single", 1, b"ABABA"),
        ("overlap", 2, b"ABABA"), ("constant", 3, b"A" * 512),
        ("binary", 2, bytes(range(256)) * 2), ("full", 5, b"ABABA"),
        ("oversized", 6, b"ABABA"), ("huge", maximum, b"AB"),
    ]:
        yield "ngrams", name, struct.pack("<Q", n) + data
    for name, left, right in [
        ("worked", b"AAAB", b"AB"), ("equal", b"AABB", b"ABAB"),
        ("disjoint", b"AAA", b"BB"), ("left_empty", b"", b"AB"),
        ("right_empty", b"AB", b""),
        ("maximum_bytes", bytes(range(256)) * 8, bytes(range(255, -1, -1)) * 8),
    ]:
        yield "total_variation", name, b"\0" + struct.pack("<Q", len(left)) + left + right
    for name, pairs in [
        ("tiny", [(maximum - 1, maximum - 2), (1, 1)]),
        ("maximum_disjoint", [(maximum, 0), (0, maximum)]),
        ("scaled", [(3, 9), (1, 3)]), ("count_empty", [(0, 1)]),
        ("overflow", [(maximum, maximum), (1, 1)]),
        ("full_support", [(i + 1, 256 - i) for i in range(256)]),
    ]:
        yield "total_variation", name, b"\1" + b"".join(struct.pack("<QQ", a, b) for a, b in pairs)
    yield "total_variation", "bounded_counts", b"\2" + b"\xff" * 4096
    for target in ["byte_entropy", "count_entropy", "ngrams", "total_variation"]:
        yield target, "short", b"\xff"
        yield target, "raw_empty", b""


def seeds():
    for target, name, data in base_seeds():
        yield target, name, data
        if target == "total_variation":
            yield "jensen_shannon", name, data
    for name, n, data in [
        ("zero_empty", 0, b""), ("zero_nonempty", 0, b"AB"),
        ("empty", 1, b""), ("single", 1, b"A"), ("overlap", 2, b"ABABA"),
        ("constant", 32, b"A" * 64), ("distinct", 2, bytes(range(64))),
        ("grouped", 2, b"AABB"), ("alternating", 2, b"ABAB"),
        ("full", 5, b"ABABA"), ("oversized", 6, b"ABABA"),
        ("huge", (1 << 64)-1, b"AB"), ("binary", 2, b"\0\xff" * 32),
    ]:
        yield "subword", name, struct.pack("<Q", n) + data
    yield "subword", "short", b"\xff"
    yield "subword", "raw_empty", b""
    for name, data in [
        ("empty", b""), ("single", b"A"), ("two_equal", b"AA"), ("two_distinct", b"AB"),
        ("overlap", b"AAAA"), ("periodic", b"ABABAB"), ("distinct", bytes(range(128))),
        ("constant", b"A"*128), ("binary", b"\0\xff"*64), ("mills", b"101101011"),
        ("estevez_rams", b"010011101101100"), ("late_novelty", b"A"*127+b"B"),
        ("forward", b"AAB"), ("reverse", b"BAA"),
    ]:
        yield "lz76", name, data
    for name, data in [
        ("empty", b""), ("single", b"A"), ("short", b"ABABAB"),
        ("window_before", bytes(range(256))*127+b"A"*255),
        ("window", bytes(range(256))*128),
        ("window_after", bytes(range(256))*128+b"A"),
        ("constant", b"A"*65536), ("late_novelty", b"A"*65535+b"B"),
    ]:
        yield "compression", name, data
    state = 0x123456789abcdef0
    data = bytearray()
    for _ in range(65536):
        state ^= (state << 13) & ((1 << 64)-1)
        state ^= state >> 7
        state ^= (state << 17) & ((1 << 64)-1)
        data.append(state >> 56)
    yield "compression", "mixed", bytes(data)
    for name, a, b, c, split, data in [
        ("empty",0,0,0,0,b""), ("zero_denominator",0,0,1,0,b"A"),
        ("negative",10,10,5,2,b"ABAB"), ("above_one",1,1,10,1,b"AB"),
        ("unit_gap",2**64-1,2**64-1,2**64-2,1,b"AA"),
        ("large_positive",1,1,2**64-1,2,b"\0\xff\0"),
        ("constant",3,3,5,128,b"A"*1024),
        ("mixed",5,3,8,255,bytes(range(256))*4),
    ]:
        yield "ncd",name,struct.pack("<QQQH",a,b,c,split)+data
    yield "ncd","short",b"\xff"
    yield "ncd","raw_empty",b""
    for name, width, step, skip, data in [
        ("empty",1,1,0,b""), ("zero_size",0,1,0,b"A"),
        ("zero_step",1,0,0,b"A"), ("both_zero",0,0,0,b""),
        ("overlap",3,2,1,b"ABCDEFGH"), ("adjacent",3,3,0,b"ABCDEFGH"),
        ("gaps",2,3,0,b"ABCDEFGH"), ("oversized",9,1,0,b"ABCDEFGH"),
        ("extreme_width",2**64-1,1,2**64-1,b"AB"),
        ("extreme_skip_step",1,2**64-1,2**64-1,b"ABCDEFGH"),
        ("extreme_skip",1,2,2**64-1,bytes(range(256))),
        ("binary",2,1,2,b"\0\xff\0\xff"),
    ]:
        actions = bytes([6,7,0,1])*8 if "extreme" in name else bytes(range(32))
        yield "windows",name,struct.pack("<QQQ",width,step,skip)+actions+data
    yield "windows","raw_empty",b""
    yield "windows","short",b"\xff"
    for name, width, skip, data in [
        ("empty",1,0,b""), ("zero",0,0,b"A"), ("zero_empty",0,0,b""),
        ("tail",3,0,b"ABCDEFGH"), ("exact",2,0,b"ABCDEFGH"),
        ("oversized",9,0,b"ABCDEFGH"), ("single",1,0,b"AB"),
        ("extreme_width",2**64-1,2**64-1,b"AB"),
        ("extreme_skip",3,2**64-1,b"ABCDEFGH"),
        ("binary",2,1,b"\0\xff\0\xff\0"),
    ]:
        actions = bytes([6,7,0,1])*8 if "extreme" in name else bytes(range(32))
        yield "blocks",name,struct.pack("<QQ",width,skip)+actions+data
    yield "blocks","raw_empty",b""
    yield "blocks","short",b"\xff"
    for name, alphabet, data in [
        ("empty",2,b""),("constant",2,b"AAAA"),("uniform",2,b"AABB"),
        ("worked",2,b"AAAB"),("partial",4,b"AABB"),("full",256,bytes(range(256))),
        ("zero_alphabet",0,b"AB"),("singleton",1,b"A"),("large",257,b"A"),
        ("maximum",2**64-1,b"A"),("support",2,b"ABC"),
    ]:
        yield "statistical_complexity",name,b"\0"+struct.pack("<Q",alphabet)+data
    for name, alphabet, counts in [
        ("regression_rare",2,[2**64-2,1]),
        ("near_uniform",2,[2**63-1,2**63]),
        ("overflow",256,[2**64-1,1]),
        ("wide_full",256,[2**56-1]*256),
    ]:
        yield "statistical_complexity",name,b"\1"+struct.pack("<Q",alphabet)+b"".join(struct.pack("<Q",n) for n in counts)
    yield "statistical_complexity","bounded",b"\2"+struct.pack("<Q",256)+b"\xff"*2048
    yield "statistical_complexity","raw_empty",b""
    yield "statistical_complexity","short",b"\xff"
    for name, n, data in [
        ("empty",1,b""),("zero_empty",0,b""),("zero",0,b"AB"),
        ("single",1,b"ABABA"),("overlap",2,b"ABABA"),("distinct",2,b"AABBA"),
        ("boundary",2,b"AABB"),("constant",2,b"AAAA"),("whole",5,b"ABABA"),
        ("oversized",6,b"ABABA"),("maximum",2**64-1,b"ABABA"),
        ("rare",2,b"A"*1023+b"B"),("diverse",4,bytes(range(256))*4),
    ]:
        yield "block_entropy",name,b"\0"+struct.pack("<Q",n)+data
    yield "block_entropy","bounded",b"\1"+struct.pack("<Q",2**64-1)+bytes(range(256))
    yield "block_entropy","raw_empty",b""
    yield "block_entropy","short",b"\xff"
    for name, order, data in [
        ("empty",0,b""),("empty_context",1,b""),("zero",0,b"ABABAB"),
        ("alternating",1,b"ABABAB"),("boundary",1,b"AAB"),("weighted",1,b"AABA"),
        ("constant",2,b"AAAA"),("one",3,b"ABAB"),("none",4,b"ABAB"),
        ("maximum",2**64-1,b"AB"),("rare",1,b"A"*1023+b"BA"),
        ("all_bytes",0,bytes(range(256))),
        ("successors",1,b"\0"+b"".join(bytes([b,0]) for b in range(256))),
    ]:
        yield "conditional_order",name,b"\0"+struct.pack("<Q",order)+data
    yield "conditional_order","bounded",b"\1"+struct.pack("<Q",2**64-1)+bytes(range(256))
    yield "conditional_order","small",b"\2"+struct.pack("<Q",1)+b"AABA"*128
    yield "conditional_order","raw_empty",b""
    yield "conditional_order","short",b"\xff"
    for name, maximum, skip, data in [
        ("empty",8,0,b""),("zero_limit",0,0,b"ABCD"),("singleton",1,0,b"A"),
        ("constant",4,0,b"AAAA"),("worked",4,0,b"AABB"),("nonmonotone",5,0,b"ABABA"),
        ("prefix",2,0,b"AABB"),("oversized",100,0,b"ABCD"),
        ("max_limit",2**64-1,0,b"AABB"),("max_skip",2**64-1,2**64-1,b"ABCD"),
        ("skip_tail",2**64-1,3,b"AABB"),("binary",4,0,b"\0\xff\0\xff"),
        ("diverse",32,0,bytes(range(32))),
    ]:
        actions=bytes([4,5,6,7,0,1,2,3])*2
        if name in ["max_skip","skip_tail"]: actions=bytes([2,3,0,1])*4
        yield "block_profile",name,struct.pack("<QQ",maximum,skip)+actions+data
    yield "block_profile","raw_empty",b""
    yield "block_profile","short",b"\xff"
    for name,x,y in [
        ("empty",b"",b""),("singleton",b"A",b"X"),("constant",b"AAAA",b"XXXX"),
        ("independent",b"AABB",b"XYXY"),("dependent",b"AABB",b"XXYY"),
        ("weighted",b"AAAB",b"XXYX"),("left_empty",b"",b"AB"),("right_empty",b"AB",b""),
        ("unequal",b"AB",b"X"),("all_bytes",bytes(range(256)),bytes(reversed(range(256)))),
        ("rare",b"A"*511+b"B",b"X"*512),("binary",b"\0\xff\0",b"\xff\0\0"),
    ]:
        yield "joint",name,b"\0"+struct.pack("<Q",len(x))+x+y
        yield "mutual",name,b"\0"+struct.pack("<Q",len(x))+x+y
    yield "joint","midpoint",b"\1"+struct.pack("<Q",2**64-1)+b"AABBXYXY"
    yield "joint","raw_empty",b""
    yield "joint","short",b"\xff"
    yield "mutual","midpoint",b"\1"+struct.pack("<Q",2**64-1)+b"AABBXYXY"
    yield "mutual","raw_empty",b""
    yield "mutual","short",b"\xff"
    for name,counts in [("near_independent",[65,63,63,65]),("nonuniform_independent",[6,3,2,1]),("series_boundary",[80,48,48,80])]:
        pairs=[(0,0),(0,1),(1,0),(1,1)]
        x=b"".join(bytes([a])*c for (a,b),c in zip(pairs,counts))
        y=b"".join(bytes([b])*c for (a,b),c in zip(pairs,counts))
        yield "mutual",name,b"\0"+struct.pack("<Q",len(x))+x+y
    k = 1 << 50
    for offset in [-2, -1, 0, 1, 2]:
        pairs = [(3*k+offset, k), (k, 3*k+offset)]
        yield "jensen_shannon", f"series_boundary_{offset}", b"\1" + b"".join(
            struct.pack("<QQ", a, b) for a, b in pairs)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    expected = {ROOT / target / (name + ".bin"): data for target, name, data in seeds()}
    for path, data in expected.items():
        if args.check:
            if not path.is_file() or path.read_bytes() != data:
                raise SystemExit(f"Seed missing or changed: {path}")
        else:
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(data)
    # Additional hand-retained regression seeds are allowed and replayed too.
    print(f"{'Verified' if args.check else 'Wrote'} {len(expected)} starting seeds")


if __name__ == "__main__":
    main()
