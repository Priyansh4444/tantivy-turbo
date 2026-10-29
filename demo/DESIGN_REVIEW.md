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

## Black, social-feed pass

The background is now black with bright blue and cyan accents. The opening explains what search does and defines the two measurements before presenting them. The query-type and algorithm scenes use everyday words, and the typography is larger. The animated diagrams still label illustrative material and identify the older fork in the region comparison. Historical benchmark sources and measured values have not changed.

## Short social cut

The video is now 32.5 seconds. It removes the Apache Lucene scene and its
20/20 claim, omits the production-tool credit, and replaces two abstract
diagrams with one explicitly illustrative, staged example. The retained
benchmark screens show aggregate results and the separately measured longer
OR result, with the historical build limitation on screen. A plain-text
transcript is in `demo/README.md`.

Two GPT-6.1 Sol reviewers assessed the previous cut, and a third reviewed a
rendered draft. Their feedback led to the concrete example, simplified
comparison screens, clearer benchmark scope and more legible labels. The
third reviewer reported missing glyphs in preview images; OCR of the actual
encoded draft at those timestamps found the complete text, indicating the
preview display rather than the encoded images was unreliable there. The
final export should still be checked independently.

## Search-story pass

The revised cut replaces decorative text with an animated collection of
documents: the opening finds matching cards, and the example follows a rare
word through to documents containing both words. Benchmark screens use
proportional time bars and smaller labels. The older October 23 benchmark
figures remain identified as historical. The full-scan check now reports the
expanded 14,400 query/configuration checks, which passed in release mode.

Self-review: **8/10 for visual explanation, 7/10 for small-screen reading**.
The search example now has a visible sequence and less text, while the source
date and methodology still need small type on the benchmark screens. The
32.5-second cut remains silent and viewers can pause for details.

## Shorter motion-led cut

The prior cut felt like slides: two similar benchmark screens and a document
example that mostly changed outlines. The new cut is 21.7 seconds, with one
benchmark scene, no persistent progress bar, a moving search pulse, and
matching cards that lift while non-matches recede. The measured comparison
uses the October 28 warm top-10 data; its bars are an animated scale.

My comparison: **motion 5/10 → 7/10; reading load 6/10 → 8/10; explanation
for a new viewer 6/10 → 7/10**. These are subjective scores, not user-test
results. Remaining limitations: the timing-method note and repository address
are small on a phone, and the clip is silent. A dedicated portrait layout
could improve mobile viewing without cropping the landscape version.

Reviewers of the first draft identified a misleading early match outline and
an unclear shorter-time cue; the final cut reserves the bright outline for
actual matches and explicitly says less time is better. The reported times
were independently recalculated from the retained benchmark summary.

## Continuous-motion cut

This revision replaces the five separate scenes with one continuous 21-second
timeline at 60 fps. The search signal travels into the document field; three
illustrative matches gather while the other documents recede. The visual field
then flows into the measured comparison and the full-scan check. Quintic easing
brings moving elements to rest smoothly, and incoming and outgoing captions
are timed separately to avoid overlapping text.

Self-review after comparing the rendered drafts: **motion 7/10 → 8.5/10;
reading load 8/10 → 8.5/10**. The first continuous draft had dark pauses and
overlapping captions at transitions; a second pass holds the result cards
longer, separates the captions, and bridges the graphics. Scores are subjective:
the benchmark caveat still needs the accompanying report, and the video is a
silent landscape export rather than a purpose-built vertical cut.

## Ranked-results pass

I compared two visual sketches: a glowing orbit around matching documents and
a ranked search-results interface. The interface is easier to understand
without search-engine knowledge, so the new cut types the query, gathers the
matching documents into numbered results, then brings two visual result
streams together for the full-scan check. The documents and titles are
illustrative. The earlier benchmark and its scope remain unchanged.

Self-review against the preceding cut: the outcome is clearer, and the text
is less crowded, although the visual still has a quiet, technical aesthetic.
I fixed a title/document collision, a layering issue that put background cards
over result text, and overlapping placeholder lines during the result reveal.
The score remains subjective; it needs real viewer feedback to establish
whether the clip meets the intended impact on a small screen.

## Apache Lucene context

The cut now includes a separate Apache Lucene (Java) comparison and names
Elasticsearch, OpenSearch and Solr as products with Lucene at their search
core. It labels the October 4 result as a 20-query, older-build comparison
and shows the geometric-mean range across process orders. This is distinct
from the newer 1,201-query lucene-rs (Rust) comparison. GitHub's code-search
engine is Blackbird, so the video does not attribute that engine to Lucene.

## Fresh storyboard

This is a new 21-second composition rather than a revision of the earlier
layout: a centered search interface leads into an animated Lucene diagram,
the lucene-rs time comparison, a correctness check and the Tantivy API pitch.
The result entries are fictional and marked illustrative. Captions fade on
separate beats to keep transitions readable; the video remains 60 fps.

Self-review: the visual hierarchy and opening are clearer than the previous
cut, and the Lucene section explains why the comparison matters. The
benchmark details still need the linked report and a user can judge the
visual appeal more reliably than a self-assigned score.

## Lucene-first comparison

The 23-second cut leads with the Java Lucene top-10 speed comparison, follows
with the measured warmed-process resident memory, and places the lucene-rs
comparison near the end. The Lucene speed band represents the two measured
process-order ratios. Memory is shown as measured RSS with the default JVM
heap and mapped pages, not as a universal memory ratio. Both Lucene scenes
identify the October 4 older build; the lucene-rs scene identifies the later
1,201-query run. No storage comparison is shown.
