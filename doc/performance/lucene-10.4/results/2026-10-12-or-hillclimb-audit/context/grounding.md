# OR hillclimb grounding and design contract

Production base0e9329dfe782187a8feda0024102e4aabf517c28, core3bab188de.
Baseline worker2efc80ae6cc57c4163cab798f50bfb35ceb085c039bc4dd4b8fe31d22d387bbf;
matching retained post-profile is copied as baseline with original receipt.
New isolated worktree search/tantivy-or-hillclimb-oct07. Root alone owns heavy
build/test/profile/timing jobs. All design packages are read-only source analysis.

Frozen ruler: same1M-doc Tantivy format11 index, complete payload identity,
1227 exact default/two-config gates and1221 timed queries. Broad301OR:103actual3+
term OR,198two-term OR; Wiki20 has4two-term ORs. Current actual-port mean ratios:
3+2.273/2.305,2term2.690/2.712,allOR2.404/2.433,overall1.293/1.305. TERM/PHRASE
already beat actualport arithmetically. No query exclusions or approximate scores.
Primary eventual target: candidate/fixedport mean query medians<1 in bothAB/BA for
allOR and3+ group, beyond self-control spread; inspect overall/nonOR regressions.
At least3 distinct mechanism attempts before declaring victory. User instruction
is to keep improving to win, not to promise language alone implies superiority.

Current3..32 route private or_maxscore has compact24-byte records in fixed global
max/cost preference order; original cursors/leaves retain incoming order. Conservative
nonsubtractive inclusiveprefix bounds; fixed local partition per physical half-open
region. A certified globally optional prefix may omit its block-end cap only when
each prefix cost>=2*max live remainder cost, to avoid prior catastrophic dense-driver
regressions. Old failed broad/global policy cases401/719/791/691 and909/817 must stay
visible. The from-zero global prefix and full local cleanup/prefix pass repeat per
region. Candidates min-scan essential suffix, clear incoming contributions, score
matching essentials, test/probe optionals in reverse pruning order, replay original
f32 leaves in f64 then onef32 cast; advance matched essential cursors. Rising callback
threshold does not repartition until the region ends. Unsafe domains fall back
before cursor mutation. Debug mirror checks and10 independent rawbit traces exist.

Actual TermScorer is~2072bytes. Shallow seek_block changes skip selection/bound
cache, not decoded doc()/freq(); actual seek must reconcile stale doc before score
oradvance. Tail selected endTERMINATED is reconciled atregionfloor; keep priorselected
lastdoc. Physical max_doc includes deleted IDs;collector rejects deletions before
heap update. max_score/cost immutable; score andbounds do not advance. Cursor-owned
seek/advance mirror update categorical. See src/query/boolean_query/or_maxscore.rs,
term_query/term_scorer.rs,postings/{segment_postings,block_segment_postings}.rs.

Latest exact baseline five expensive3+ profile:86.35%self in inlinedBooleanWeight
for_each_pruning,5.91%block_max_score,2.32%load_block,no lost. Annotate localperiod
8.89%prefix store at299b5f,2.12%loop inc299b65; both REGION cleanup/prefix construction,
not per-candidate replay. 29a38dcompare3.34%,candidate essential loop/scoring around
29a18e,29a23a,optionalbound bit arithmetic29a2b5. Optimized samples are hypothesis
signals, not exact causal cycles nor a proof of speedup. Full annotate retained.

Two-term path in block_wand_union.rs::two_term_or_maxscore warms128candidates with
f32 sum, switches toblock_wand_intersection once bothglobalmax<=threshold, otherwise
single essential term walk with optionalglobalmax test; ifnoessential afterwarmup
fallsback block_wand. It does not use the new3+ MaxScore local partition. Do not assume
routing two clauses through generic newengine is faster without an isolated trial.
Baseline source+outputs must be frozen before implementation.

Port actualsource search/lucene-rs-idf-fix-oct06/src/search/{bulk,term_scorer}.rs:
MaxScoreBulkScorer compact wrappers, localmax/cost repartition, threshold crossing
repartition, top/top2 heap, single-essential batched kernel, multipleessential
windowbitmaps, richerhierarchical impacts andpooledWindowScratch80.5KiB plusbuffers.
TermScorer::next_docs_and_scores does two-passnormgather then arithmeticvectorloop
fromblockpostings. Our scoreralready has block_cursor/fieldnorm_reader/bm25_weight
internalborrow helpers; decoded docs/freqs arrays exist but SegmentPostings.cur is
private. A batch API must own cursor advancement coherently and follow exact leaf
arithmetic (see bm25.rs::score). Port accumulation order cannot be copied blindly:
full original-order f64 contract, numeric fallback, midpoint/boost/signed tests matter.

PriorB2 scalar/TF-cap tighter same-format bounds failed toreachdense configuredsingle
term opportunities; not evidence for newBoolean thresholds, but avoid redoingthat
unrelated failedunit. Eager explanationclone elimination andcompactstate alreadydone.

Design artifact: callers first, private type/signature/module sketches andpseudocode,
exactcursor/arithmetic/deletionproof, branch/complexity/scratchcost, rootcausalmechanism,
likelyrejectcases andtight firstisolatedattempt. Atleasttwo structurallydistinct designs.
No production edits/builds/benchmarks in designagents. Root willreadall andfreshjudge.
Rubric1..5: exactpublishedbits+safeproof; causalpotentialtohalvecost ratherthan5%tweak;
compactness(noindexbits/perregionallocation,boundedtemporaryscratch); smallreadable
ownership/API; verifiableindependentgates andpreservednegativecontrols/regressions.
