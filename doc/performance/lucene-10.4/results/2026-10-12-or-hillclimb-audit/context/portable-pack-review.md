# Portable packer — source-only Root handoff

Held source: `portable-pack.py`, 526 lines / 40,380 bytes, SHA256
`80df2e4cd7cf32f18acaf6f5044073718577d73dd601a6f64f021616859547a3`.

No packer, compressor, audit, Python source execution, correctness test, benchmark,
engine, build, browser, commit or push was run by this author. Reads, source edits,
and static SHA/line-count checks only. Root must review every actual source line,
execute assembly while its engine/build jobs are idle, inspect the populated
descriptor, and run the real standalone and clean committed-archive audits.

Workflow steps: trace completed from the full grounding document, original full
auditor/README/schema and final changed auditor sections; architect skipped because
the descriptor shape is already fixed by the independently authored auditor;
blocking first step was schema grounding; independent workstream was this one
packer file, separate from the auditor's files; shared writes avoided; smallest
safe decomposition is one code owner plus Root review. Nested delegation is
forbidden by the task. Runtime verification/shipping remain Root-owned.

Run from its **original** OUT location, with a new destination:

```text
python3 /home/pronsh/Coding/playground/search/bench/lucene-rs-or-hillclimb-oct07/portable-pack.py /absolute/new/packet-directory
```

The copied packer is provenance source, not a portable reassembly command: it
depends on the original local archives and Git objects. The resulting auditor is
portable and reads relative bindings only. The packer never calls a copied
controller or an engine; its only subprocess is read-only `git show`.

## Grounded coverage

- Completed attempts follow the **actual decision TSV order**. Thus 14 precedes13,
  and newly completed15 is included automatically. Future completed archives are
  derived in the same way; source-only16 and other actual pending attempts retain
  available artifacts without invented runtime claims. Pending candidate/parent
  descriptors stay null until a completed guarded decision exists.
- Original archive files retain their logical filenames, bytes, full raw telemetry,
  schedules, samples, protocols, stderr, RSS and perf traces. Large files use
  deterministic gzip with mtime0, declared by manifest encoding **without renaming
  the logical filename**. `attempts.json`, README and auditor stay identity encoded.
- Stored/raw lengths and SHA256, original-copy origins, complete inventory and
  explicit exclusions/derivations are retained. No historical receipt or original
  patch is rewritten to change absolute provenance paths.
- Stage roles/commands/environment/test totals/statuses come from actual retained
  command/receipt/logs. Failed08/09/10/12 revisions remain separate. Both10 pilots,
  full13/14 parent lanes, rejected15 pilot and absent actual-port lanes are explicit.
- Parent descriptors bind their unchanged original bytes and relocated build/gate
  proofs. Accepted chain01/03/06/11 is selected from each guard, rather than guessed
  from directory or build HEAD. The final source snapshot is dirty candidate bytes,
  distinct from a later accepted commit.
- Changed source files are copied only when their actual content SHA matches the
  selected snapshot, using archived raw source, matching held tree or Git object.
  Root explicitly authorized exact original-patch reconstruction for missing08
  failed raw source: preserve literal original patch and Git base bytes, apply exact
  context/hunk counts, require final guarded SHA and label the output reconstructed
  without a copied-origin claim. Missing/mismatching source stops assembly.
- Some failed run-specific patches09/10/12 were not retained. Their readable
  candidate descriptor diffs are explicitly derived from exact retained source and
  preserved Git base; other original historical patches remain unchanged in the
  archive. The derivation is in packing-receipt, not mislabeled as an original copy.
- Profile01 remains one unavailable original, one excluded candidate overlapping
  own Clippy, and two recreated clean profiles. Original08 full-query casefile was
  found (`attempt-08/profile-or3-all.json`) and is hash bound. The source supports
  explicitly disclosed hash-matched case reconstruction only if an actual original
  file is missing; it does not invent a matching digest.
- Full fixed-port source snapshot/harness is preserved; executable Rust sources and
  package configurations are copied with the snapshot's exact hashes. Unrelated
  fixture payload is not copied merely to duplicate the port repository.

## Root's material checks before publication

1. Review all source and inspect the resulting `attempts.json` against actual
   archives, including failed snapshots and the source reconstruction receipt.
2. Run the populated packet's `recompute.py` with stdout redirected **outside** the
   packet. An audit-output file inside violates its exact inventory.
3. Exercise malformed inventory/source/parent/schedule/checksum/READY handling on
   separate copied fixtures, keeping published artifacts immutable. The author
   has not executed any of these checks.
4. Confirm all pending work is represented as pending and all newly completed work
   has been archived and appended to the decision trail before packing. A changing
   decision trail aborts the packer; a partial destination is retained for review,
   never overwritten automatically.
5. Review README provenance wording, add the distinct clean Git-archive wrapper
   if desired, and audit the exact committed packet bytes separately before push.

## Limits

This is assembly tooling, not new performance evidence. Packet integrity cannot
authenticate an experimenter, prove source-to-binary compilation, recover lost
original01 profile data, validate absent large index/corpus/binary payload, establish
causal cycles, remove background load, establish configured Java/port timing, prove
universal wins or establish all Lucene public API parity. Source reconstruction is
byte-exact against a historical guard but is explicitly reconstructed evidence.
The current actual-port target remains whatever the real latest accepted lane
computes; assembly does not force it true.
