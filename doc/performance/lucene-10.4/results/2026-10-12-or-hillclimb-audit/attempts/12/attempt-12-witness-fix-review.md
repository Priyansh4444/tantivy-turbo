# Independent review of attempt 12 witness corrections

No blocker found in the two test-only corrections. The previous production
review remains applicable; root must rerun the targeted tests and later gates.

Reviewed the complete small witness-fix patch, receipt, explanation, both actual
resulting functions, and retained first-run stdout/stderr/receipt. That run
compiled 23 tests, passed 21, and failed only the required nonempty partial-batch
observation and the required widening observation in the midpoint fixture.
Those failures occurred after the shared canonical trace and literal enclosure/
progress checks. The old log did not identify the failing length/order/threshold;
the specific TF1-tail/equality causes are grounded source explanations, not
additional telemetry from that log.

## Independently checked identity and isolation

- Base accepted11 `43f06833a325d2a18c03bd7e4fa65f2f27944292`.
- New source SHA256
  `a53374a75b5dbedb3e8f148b94f8da92a6d4383142e69cc8b50a92330c2f96d1`.
- Retained failed source SHA256
  `6c71f32394bac8d1c5998c511585904e09ac68757063b2f2bc96afb375258d1d`.
- Exact replacement of only the two corrected functions by their retained old
  bytes reconstructs the entire old source, byte-for-byte and with its old SHA.
- The entire prefix before the tests module, including production and cfg(test)
  observers, is independently byte-identical. All other fixtures and helpers
  consequently remain byte-identical; no acceptance rule or production route
  was changed to make the observations pass.

## Competitive partial batch

Every partial final driver block now contains a last TF30 leaf. With unchanged
norms/weight this is stronger than the earlier TF20 leaf that can raise Top1's
threshold, so a later partial block can remain competitive. For length129 the
last leaf is doc382 itself and becomes TF30 rather than TF20; it still exercises
the exclusive unread head and later winning callback. Full-only lengths128,
256 and768 are unchanged. The nonempty partial-batch assertion remains required
for every other length, now with a useful length diagnostic. Actual widening,
sole suffix, the 128-row batch retaining head382/up_to382, and complete canonical
trace/enclosure/progress assertions are all retained.

This fixes corpus realizability while preserving the required observation. It
does not replace a missing observation with a weaker assertion or derive the
expected scores from the candidate implementation.

## Midpoint strictness during actual expansion

The new TF2 leaf at doc381 is in the driver's first 128-posting full block and
exceeds doc0's TF1 leaf under the unchanged literal scorer. Doc0, all tiny leaves,
all incoming-order permutations and sequential f64 expected midpoint calculation
remain unchanged. The stronger same-block leaf makes the first driver bound
competitive even when the initial threshold rejects doc0, allowing the fixture
to require an actual expansion at equal/neighbor thresholds.

The old all-empty expectation is correctly replaced by explicit absence of doc0
at equality/above, because the corpus now deliberately contains a stronger later
document. Below equality, the first callback must still be doc0 with the exact
incoming-fold raw bits. Doc381 is explicitly required for every tested incoming
order, neighboring threshold, and fixed/Top1/Top10 mode. Full BufferedUnionScorer
callback/raw-score equality and independent literal certificate/cursor evidence
remain the overall score oracle.

## Limits

This reviewer ran only lightweight file/diff/hash checks. No compiler, test,
worker, profile, latency, compression or browser job; no source edits, children,
commits or pushes. Only this report was written. The corrections make the
required witnesses plausible and preserve oracle strength; their actual runtime
realization is still root's responsibility. No performance result follows.
