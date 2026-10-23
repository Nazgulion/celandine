# Jensen–Shannon reference notes

## Sources and convention

Jianhua Lin (1991), *Divergence Measures Based on the Shannon Entropy*, IEEE
Transactions on Information Theory 37(1), 145–151, DOI
[10.1109/18.61115](https://doi.org/10.1109/18.61115),
[paper](https://www.cise.ufl.edu/~anand/sp06/jensen-shannon.pdf).
Lin describes the named divergence, including unequal weights and more than two
laws. Celandine implements only two laws with equal weights and base-2 logs.

Dominik M. Endres and Johannes E. Schindelin (2003), *A New Metric for Probability
Distributions*, IEEE Transactions on Information Theory 49(7), 1858–1860,
DOI [10.1109/TIT.2003.813506](https://doi.org/10.1109/TIT.2003.813506),
[paper](https://pdodds.w3.uvm.edu/research/papers/others/2003/endres2003a.pdf).
Their squared expression is twice the equal-weight divergence. Rescaling its
square root by a positive constant gives the distance convention used here.

The divergence is `J(P,Q)=H((P+Q)/2)-(H(P)+H(Q))/2` in bits, and the separately
named distance is `sqrt(J)` in square-root bits. The mixture weights do not
depend on sample sizes. Empty empirical states are rejected, including two
empty states; zero counts inside nonempty laws are allowed. No source model,
weight adjustment, normalization, or smoothing is applied.

## Stable evaluation derivation

For each label put `s=p+q` and `d=|p-q|/s`, skipping `s=0`. Substitution of
`p=s*(1+d)/2`, `q=s*(1-d)/2` into the defining KL-to-mixture expression gives
`s/2 * g(d)`, with
`g(d)=((1+d)ln(1+d)+(1-d)ln(1-d))/(2 ln(2))`.
Swapping `p,q` changes no result. A zero probability gives `d=1` and `g(1)=1`.

Since `g'(d)=atanh(d)/ln(2)`, integrating its geometric power series gives
`g(d)=sum_{k>=1} d^(2k)/(2k*(2k-1)*ln(2))`. At `d<=1/2`, all terms are
nonnegative and the tail beyond 32 terms is below `1e-21` relative to the first
term. The implementation can stop sooner when adding a term leaves the `f64`
sum unchanged. At larger `d`, the `ln_1p` expression is well conditioned; the
endpoint is handled explicitly to avoid `0*ln(0)`. These are algebraic evaluation
choices, not a new divergence definition or a claim of exact floating arithmetic.

For empirical counts `a,b` and positive totals `N,M`, use exact `u128` products
`A=a*M`, `B=b*N`. Form the difference before conversion; form the sum after
conversion to avoid overflow at `usize::MAX` totals. Then
`s/2=(A+B)/(2*N*M)` and `d=|A-B|/(A+B)`. This gives exact equality detection for
proportional count tables and retains differences that rounded probabilities
can hide. Compensated summation combines the per-label contributions.
Disjoint supports give one explicitly. The code does not clamp other outputs.

## Independent verification

`scripts/reference_jensen_shannon.py` builds exact `Fraction` probabilities and
mixtures, evaluates the defining signed logarithmic terms at 160 decimal digits,
and exports divergence and square-root distance. Its 171 pairs include all 148
TV count cases plus a worked example, cases near the series boundary, large
majority products, and nearly identical laws. Reusing count cases does not reuse
the Rust evaluation algorithm. Tests compare nonzero results at `1e-12` relative
tolerance and zeros exactly; fixtures too wide for a target's `usize` are skipped.

Properties check an independent entropy-of-mixture identity, symmetry, equality,
repetition, ordering, shared relabeling, and the distance triangle inequality.
For additional cross-checks they use `TV^2/(2 ln(2)) <= J <= TV`, with base-2 `J`
and the half-L1 TV convention. Fuzzing also exercises invalid count imports,
empty samples, tiny positive distances, and both byte and distribution APIs.

The initial property oracle used ordinary summation for each entropy and drifted
by about `2.25e-14` on a retained case. Independent 160-digit arithmetic gave
`0.3272409204694447`; the naive oracle gave `0.3272409204694222`. The oracle now
uses pairwise summation; its tolerance was not loosened. The reproducible seed
in `tests/jensen_shannon.proptest-regressions` failed before that correction and
passes afterward. No library algorithm change was needed for this oracle issue.

See the [divergence guide](../divergence/jensen_shannon.md) and
[distance guide](../divergence/jensen_shannon_distance.md) for worked examples,
units, complexity, and limits. Public KL divergence, arbitrary probability
vectors, adjustable weights/bases, and generic alphabets remain future work.
