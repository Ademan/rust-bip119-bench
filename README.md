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
    Finished `bench` profile [optimized] target(s) in 0.04s
     Running unittests src/lib.rs (target/release/deps/bip119_bench-ed0238849b35ad35)

running 1 test
test test::test_equivalence ... ignored

test result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running benches/bench_hashing.rs (target/release/deps/bench_hashing-afd48536593d3759)
CTV Template Calculation 2-in-3-out/from_transaction
                        time:   [508.41 ns 508.75 ns 509.08 ns]
                        change: [−0.0699% +0.1386% +0.3217%] (p = 0.18 > 0.05)
                        No change in performance detected.
Found 4 outliers among 100 measurements (4.00%)
  2 (2.00%) high mild
  2 (2.00%) high severe
CTV Template Calculation 2-in-3-out/from_components
                        time:   [348.69 ns 348.92 ns 349.17 ns]
                        change: [−0.6370% −0.5532% −0.4676%] (p = 0.00 < 0.05)
                        Change within noise threshold.
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
```

In this run `from_components` is 31.4% faster than `from_transaction`, and these
numbers are representative of my previous observations with very little variance.
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
I am satisfied for now knowing that `from_components` is significantly faster
and don't intend to investigate exactly why.
