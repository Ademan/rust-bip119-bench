# Overview

This repo contains a quick benchmark to evaluate whether the
`DefaultCheckTemplateVerifyHash::from_components` API is worth the headache.
In most cases it won't be, however in my
[More Complicated CTV Vault](https://github.com/lnhance-expedition/mccv) project,
I generate tens of thousands of CTV hashes and this can take minutes for
sufficiently large vaults.

# Results

On my machine with a "AMD Ryzen 7 5700U with Radeon Graphics" and 24GiB RAM, I
consistently observed a ~31% performance improvement with `from_components`.
Furthermore, I suspect the benefits could be significantly greater in
`More Complicated CTV Vault`s because generates templates in parallel, and
rust's default allocator will have to deal with thread contention.
I'm satisfied that this validates the usefulness of the API, though it is
cumbersome and could probably be improved.

```
     Running benches/bench_hashing.rs (target/release/deps/bench_hashing-afd48536593d3759)
CTV Template Calculation 2-in-3-out/from_transaction
                        time:   [512.98 ns 513.76 ns 514.80 ns]
                        change: [−0.3231% −0.2120% −0.0747%] (p = 0.00 < 0.05)
                        Change within noise threshold.
Found 8 outliers among 100 measurements (8.00%)
  6 (6.00%) high mild
  2 (2.00%) high severe
CTV Template Calculation 2-in-3-out/from_components
                        time:   [349.64 ns 349.87 ns 350.11 ns]
                        change: [−0.5524% −0.4694% −0.3857%] (p = 0.00 < 0.05)
                        Change within noise threshold.
Found 1 outliers among 100 measurements (1.00%)
  1 (1.00%) high mild
CTV Template Calculation 2-in-3-out/from_components_convenient
                        time:   [347.52 ns 347.75 ns 348.01 ns]
                        change: [−0.7441% −0.6353% −0.5312%] (p = 0.00 < 0.05)
                        Change within noise threshold.
Found 5 outliers among 100 measurements (5.00%)
  5 (5.00%) high mild
```

In this run `from_components` is 31.9% faster than `from_transaction`, and these
numbers are representative of my previous observations with very little variance.
Furthermore `from_components_convenient` is not only on-par with `from_components`, but it is consistently 0.6% faster than `from_components`.
I'm not certain why this is, but that is a pretty negligible savings.
More importantly `from_components_convenient` is meant to be, and succeeds at being similarly fast and less error prone.
There are three things I attribute the performance improvement to in decreasing
order of importance.
Most importantly, I attribute the bulk of the performance improvement to avoiding
allocating `ScriptBuf`s for the outputs.
This eliminates two 34 byte allocations and one four byte allocation.
Secondly, the `from_components` code avoids allocating two `Vec`s for inputs and
outputs.
The allocations themselves are for insignificant amounts of memory, but the
allocator is not free, and this seems to be reflected in the performance numbers.
Finally, we also avoid iterating over the `script_sig`s to check if they're
non-empty, because we know they are.
I'm skeptical that this has any impact whatsoever, iterating over a 2 item `Vec`
should be nearly free.
There are other possibilities that would undermine my conclusion, but they
strike me as extremely unlikely.
Furthermore, the addition of `from_components_convenient` rules out some of these ideas like the manual unrolling of the loop.
I am satisfied for now knowing that `from_components` is significantly faster
and don't intend to further investigate exactly why.
