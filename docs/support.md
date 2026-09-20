# Toolchain, platform, and compatibility policy

## Initial supported toolchain

Celandine's minimum supported Rust version is **1.90**. This is the supported
baseline, not a claim that every earlier compiler fails to compile the library.
The crate uses edition 2024. `Cargo.toml` declares `rust-version = "1.90"`, and
CI installs Rust 1.90.0 and checks all targets, examples, tests, and documentation.

This baseline was chosen from successful local and CI validation. A lower
minimum requires separate testing. Raising the minimum requires a documented
compatibility decision and changelog entry; it is not a routine patch change.
Newer Rust releases are not automatically evidence of tested compatibility.
The pinned CI toolchain is authoritative for the current supported baseline.

The library has no runtime or build dependencies. Criterion and proptest are
development dependencies, and the committed lockfile records the validated
development environment. Ordinary Rust tests use committed reference fixtures;
Python is needed only to regenerate or independently check them. Reference
scripts require Python 3.10 or later; this review ran them on Python 3.10.12,
and CI uses Python 3.12.

See Cargo's [Rust-version documentation](https://doc.rust-lang.org/cargo/reference/rust-version.html)
for the meaning of the manifest field.

## Platform support and evidence

The initial tested runtime target is **`x86_64-unknown-linux-gnu`**. GitHub CI
runs on Ubuntu 24.04. Local validation also exercises this target, including
debug/release numerical tests, allocations, examples, and package verification.

The readiness review additionally ran `cargo check --locked --lib --target ...`
on the installed Rust 1.90.0 standard libraries for these targets:

| Target | Evidence | Runtime/numerical support |
| --- | --- | --- |
| `aarch64-unknown-linux-gnu` | Library type-check/code-check passed | Not verified |
| `aarch64-unknown-linux-musl` | Library type-check/code-check passed | Not verified |
| `armv7-unknown-linux-gnueabihf` | Library type-check/code-check passed, including 32-bit `usize` | Not verified |
| `wasm32-wasip1` | Library type-check/code-check passed | Not verified |

These checks do not link or execute applications. Windows, macOS, other targets,
and cross-target test/benchmark builds were not validated in this review. A
platform becomes supported after its runtime suite and numerical conventions
have been verified and repeatable CI coverage is established. Wider imported
count fixtures are explicitly skipped when their totals do not fit `usize`;
that accommodation alone does not establish 32-bit runtime support.

## Numerical compatibility

Determinism means repeated calls with identical input and parameters in the
same binary/runtime return identical results. There is no promise of bitwise
identity across toolchains, architectures, or math libraries. The documented
test tolerances are bounded validation criteria, not universal error bounds.

The contracts in [mathematical conventions](mathematical-conventions.md) and
[numerical behavior](numerical-behavior.md) govern units, valid domains, errors,
empty input, allocation behavior, and precision limits. A correctness fix may
change low-order floating-point bits and must carry independent verification,
any relevant regression tests, and a changelog entry. Promised equivalences,
such as explicit Shannon base two matching default Shannon, remain tested.

## Versioning and release status

The package version is currently `0.1.0`, with unreleased changes recorded in
the changelog. That number alone is not evidence of a published release.
Publishing stays disabled until the [v0.1 readiness gates](releases/v0.1-readiness.md)
are resolved and a release is explicitly authorized.

Once v0.1 is released, preserve public signatures and documented conventions
within `0.1.x`; reserve incompatible API, domain, unit, ownership, or support-policy
changes for a minor-version decision. Additions and documented correctness fixes
may be patch releases when they preserve those contracts. Do not promise stable
v1.0 behavior during this early series. Cargo's [version compatibility rules](https://doc.rust-lang.org/cargo/reference/semver.html)
provide the package-compatibility reference.

The owner selected MIT only. The repository's [LICENSE](../LICENSE) contains
the license text, and the crate declares `license = "MIT"`. See the canonical
[MIT license](https://spdx.org/licenses/MIT.html) for its SPDX identifier and text.
