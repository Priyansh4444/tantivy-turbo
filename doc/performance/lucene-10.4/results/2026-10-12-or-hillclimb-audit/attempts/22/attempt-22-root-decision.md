# Root decision for attempt 22

Keep a partial OR win. Root reviewed both actual source diffs and every added
numeric test, then read the completed frozen parent and actual-port results.
There is no concrete correctness blocker: the original multiplication remains;
the exact midpoint cutoff accounts for the f64 successor, f32 ties-to-even,
signed zero, negative thresholds, infinity, NaN and the finite MAX endpoint.
Only the optional probe predicate changes. The cutoff refresh immediately follows
every callback assignment. Existing canonical callback/cursor fixtures remain.

Source identities:
- OR: 590f6a2b0f493fe2c0b6ab7c17a41ab610c95e50a76c8f4a6ff42f44e544806e
- score combiner: 2679f2aab4434a4f4b75456185fee01d03506b5d40fb95f519e135664068997e
- measured worker: 005dd61a74e1885cb3e145987fc67399d89fe662bb2409aa00a9c1d174726b84

13 OR and 8 numeric targeted tests pass; the full suite has 1446 passed and
7 ignored. All three 1227-query exact-result profiles pass. Clippy and both
source formatting checks pass. Configured profiles establish own-oracle
correctness only. The optimized route retains its existing monotone-threshold
domain, original term count, exact score order and index format.

Longer OR arithmetic means take 2.768%/3.134% less time in AB/BA, past
0.3607% group control log spread. All OR also wins in both orders past controls.
55 longer OR queries win beyond their controls; none lose. No other group
loses in both orders past group controls. This does not establish a whole-suite
win: ALL is mixed, PHRASE is especially variable, and all 60 full-suite
per-query controlled losses remain retained, including perfume bottles
(1.1973/1.1715 ratios) and movie theatres (1.2074/1.2281).
No causal explanation or exclusion is assigned to those losses.

The actual fixed-port target is false. Current longer OR candidate/port ratios
are 1.7081/1.7237 and all OR ratios are 1.4732/1.4866. This is a separate
contemporaneous comparison, not a source speedup inferred from historical
port ratios. Binary and text size increase by 448 bytes, with unchanged
rodata. One f64 cache is 8 logical bytes, with no new allocation or index
change; compiled stack/register allocation and an RSS advantage are unproven.
Benchmarks ran under the user's background load without pausing applications.
