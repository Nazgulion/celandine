# Fixed-block fuzz campaign

On 2026-10-24, **100,000 executions passed** in one reported second with no
sanitizer or assertion failures. Final counters: `cov=105`, `ft=335`, RSS 189 MiB
(including the harness and sanitizer). Counters are not line percentages; run
counts include corpus initialization and are not distinct input counts.

Two little-endian u64 values select block width and a wide skip, followed by 32
operation bytes and up to 256 payload bytes (304 total). Short headers are
zero-padded; excess payload is ignored. Operation modulo four selects next,
next_back, nth or nth_back. Bit 2 selects the header skip; otherwise the skip is
the operation byte shifted right three bits.

The oracle independently enumerates positions, selects multiples of width where
a full block fits, and puts those slices in a deque. Its total selected length
locates the original tail. Mixed operations must agree on values, addresses,
remaining count, exact size hints, cloned traversal, and unchanged tail bytes
and address. Zero width returns the documented error. Twelve seeds include
empty/short headers, invalid size, exact/partial/oversized blocks, binary bytes,
and maximum width/skip values. Stable replay covers 164 generated seeds plus
one retained TV oracle regression across eleven targets.

```sh
python3 scripts/fuzz_campaign.py --target blocks --runs 100000
```

Settings: nightly-2025-10-25, cargo-fuzz 0.13.1, libfuzzer-sys 0.4.10,
AddressSanitizer, debug assertions and overflow checks; seed 20260920, one worker,
maximum length 304, ten-second per-input timeout, 2048 MiB RSS limit. The campaign
runs outside the restricted sandbox for LeakSanitizer process inspection, with
sanitizers enabled. Runtime evidence targets Linux x86-64.

- [Manifest](blocks-manifest.json): versions, commands, source and seed hashes.
- [Compressed log](blocks.log.gz).

Parent commit: `168fdab`; hashes identify tested working-tree source. Archived
checkout paths use `$REPO`. CI runs 10,000 executions per target. No defect was
found in this increment; normal tests include maximum skips to cover the edge
case discovered during sliding-window work. Independent Python byte accumulation
and full-input reconstruction tests supplement fuzzing. This bounded campaign
does not establish exhaustive coverage.
