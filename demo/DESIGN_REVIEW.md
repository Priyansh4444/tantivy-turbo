# Visual review

Self-review, October 28. The criteria are drawn from the referenced notes work: a short reading path, diagrams and numbers beside their explanation, consistent alignment, calm dark styling, and motion that explains one change at a time. Scores are my judgment, not an independent user study.

| Criterion | Before | After | Evidence and remaining issue |
|---|---:|---:|---|
| Reading order and hierarchy | 7/10 | 8/10 | Benchmark titles and labels are clearer; the video still contains several dense comparison scenes. |
| Measurement clarity | 6/10 | 8/10 | The video marks 1× as equal speed and shows logarithmic ticks. Dates and older-build qualifications remain visible; some details still require the benchmark document. |
| Motion | 6/10 | 8/10 | Scene titles no longer overlap. The 0.4-second transition fades through the background, which is calm but briefly creates a dark frame. The longer holds give charts more reading time. |
| Small-screen readability | 4/10 | 7/10 | The primary lucene-rs TOP-10 chart has a portrait layout for narrow screens. Other wide charts and small video footnotes still benefit from zooming on a phone. |
| Consistency and restraint | 7/10 | 8/10 | Removed repeated row dividers and preserved a small set of type and color roles. The other visualizations still need their own phone layouts. |

Overall: **6/10 → 8/10**. I would not call this a 10: the rest of the benchmark charts need responsive layouts, and a video cannot offer pause, reduced-motion and scrub controls inside the frame itself. A next pass should build a responsive, accessible benchmark view around the source data and provide motion controls for the explanatory diagrams.

The checked-in chart and video numbers remain the historical results labeled in the figures. This is a presentation change, not a new benchmark or a performance claim. Regenerate charts with `python3 bench-results/make_charts.py`; render the video with the commands in `demo/README.md`.

## Cooler palette and motion pass

The dark visuals now use a deeper navy surface, cool text, blue for TOP-10 and cyan for COUNT. The OR diagram includes a moving scan marker and growing region blocks; the AND windows grow into place, and the intro settles without an abrupt opacity change. The changing counters show the measured or illustrative endpoints rather than inventing intermediate observations. The chart values and their source dates are unchanged.
