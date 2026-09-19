# Tsallis entropy

## Definition

Tsallis entropy is a family of probability-based measures indexed by an order
`q`. This API applies it to empirical byte frequencies with a fixed scale chosen
to recover Shannon entropy in bits at order one.

## Short history

Constantino Tsallis proposed this entropy in 1988 as a generalization of
Boltzmann–Gibbs statistics, inspired by multifractal probability sums. His paper
allows a conventional positive scaling constant and recovers the natural-log
entropy as `q` approaches one ([original paper](https://doi.org/10.1007/BF01016429)).
Here the formula is used as a domain-independent measure of finite distributions.

## Why it matters

Changing `q` changes sensitivity to the distribution of probabilities. At order
two it measures the chance that two independent draws differ, up to the chosen
scale. Its composition rule differs from additive Shannon entropy, providing
another way to describe diversity. It does not measure sequence structure.

## Formula

```text
S_q = (1 - sum_i p_i^q) / (q - 1) / ln(2)    (q != 1)
p_i = c_i / n
S_1 = -sum_i p_i * log2(p_i)
```

`i` indexes observed byte symbols, `c_i` is their count, and `n>0` is sample size.
`p_i` is the empirical probability, and `q` is a finite nonnegative order.
The sum raises each positive probability to `q`. The factor `1/ln(2)` fixes
**Shannon-bit scaling**: the limit at one agrees with base-2 Shannon entropy.
It is a constant scale, not normalization to `[0,1]` or division by support size.
The unscaled Tsallis formula instead tends to natural-log Shannon entropy.

## Interpretation

For a fixed law, the result is nonincreasing with `q`. For a fixed positive
order, uniform probabilities maximize it on a given support. Unlike Rényi,
uniform distributions generally have different values at different orders.
For independent product laws, the composition rule in this scale is
`S_q(P*Q)=S_q(P)+S_q(Q)+(1-q)*ln(2)*S_q(P)*S_q(Q)`.
Consequently the result is generally not an additive mean description length.
No independence assumption is made about the input sequence itself.

## Input domain

Any byte slice or validated count-backed distribution, with finite `q>=0`.
Negative zero is treated as zero. Negative orders, NaN, and either infinity
return [`crate::entropy::InvalidTsallisOrder`], even for empty or constant input.
The byte entry point validates before counting. Arbitrary probability vectors,
smoothing, bias correction, and other scale parameters remain outside this API.

## Output range

For nonempty support of size `k`, `0 <= S_q <= U_q(k)`, where
`U_q(k)=(1-k^(1-q))/(q-1)/ln(2)` at `q!=1`, and `U_1(k)=log2(k)`.
Across byte distributions and accepted orders, the maximum is `255/ln(2)`,
approximately 367.887. The usual 8-bit Shannon bound does not apply to this family.
Rounding can slightly exceed a mathematical bound; outputs are not clamped.

## Exact vs estimator

This is the exact mathematical functional of the empirical law, evaluated with
approximate `f64` arithmetic. As an estimate of an unknown source's Tsallis
entropy it has finite-sample effects and no confidence guarantee. It captures
neither unseen symbols nor temporal dependence nor an entropy rate.

## Edge cases

Empty input returns positive zero by project convention for valid orders and
has no empirical probability law. Constant input returns positive zero. At
`q=0`, only observed support counts: `S_0=(k-1)/ln(2)` for `k>0`. Zero-count
symbols never contribute, so `0^0` is not evaluated. At exact `q=1`, return the
existing Shannon calculation. Near-one values keep their supplied order.

## Computational complexity

Byte input takes `O(n+256)` time and fixed `O(256)` histogram storage. Reusing a
distribution takes `O(256)` time and constant auxiliary storage. Neither path
allocates on the heap; the distribution path does not recount the sequence.

## Numerical notes

Write `t=q-1` and use `1-sum(p_i^q)=-sum(p_i*expm1(t*ln(p_i)))`.
`expm1(x)` computes `exp(x)-1` accurately near zero, avoiding cancellation near
one. A dominant symbol's logarithm uses the integer complement `n-c_i` before
conversion to `f64`, preserving tiny positive results when `c_i/n` rounds to one.
For `0<q<0.5` and counts at most half the total, each numerator term instead
uses `p_i-exp(q*ln(p_i))`: this avoids amplified roundoff near zero and cannot
suffer severe subtraction cancellation in that region. The order is unchanged.
Compensated summation uses deterministic byte order. The numerator is summed
before division by `t` and `ln(2)`, retaining small contributions before final
scaling. Large negative exponents may saturate `expm1` to minus one; final values may be subnormal.
At exact one, Shannon's documented relative-accuracy limitation is inherited.
Tests use `1e-12` absolute tolerance plus relative checks on small positive values;
these are acceptance thresholds, not universal error bounds. Cross-platform
bitwise identity is not promised.

## Example

For `AAAB`, counts are 3 and 1, so the order-two power sum is
`(3/4)^2+(1/4)^2=10/16`. Hence `S_2=(1-10/16)/ln(2)=0.541011` approximately.
This represents a differing-draw probability of `6/16`, divided by `ln(2)`.
The order-one value is Shannon entropy, approximately `0.811278` bits per symbol.

```rust
use celandine::entropy::{shannon, tsallis_entropy};
assert!((tsallis_entropy(b"AAAB", 2.0).unwrap() - 0.375 / std::f64::consts::LN_2).abs() < 1e-12);
assert_eq!(tsallis_entropy(b"AAAB", 1.0), Ok(shannon(b"AAAB")));
assert!(tsallis_entropy(b"", f64::INFINITY).is_err());
```

## References

C. Tsallis (1988), *Possible Generalization of Boltzmann–Gibbs Statistics*,
Journal of Statistical Physics 52, 479–487, equations (1)–(3).
[DOI](https://doi.org/10.1007/BF01016429),
[original paper scan](https://physicsgg.me/wp-content/uploads/2023/08/possible_generalization_of_boltzmann-gibbs_statist.pdf).
We select its positive scaling constant as `1/ln(2)` and restrict the parameter
to finite nonnegative orders with an observed-support convention at zero.

## Related measures

[`crate::entropy::shannon`] is the order-one limit. For Rényi entropy `H_q`,
`S_q = (2^((1-q)*H_q)-1)/(1-q)/ln(2)` when `q!=1`.
Collision entropy at order two is the negative logarithm of the matching
probability; Tsallis at two instead scales its complement. These quantities
are related but not interchangeable.
