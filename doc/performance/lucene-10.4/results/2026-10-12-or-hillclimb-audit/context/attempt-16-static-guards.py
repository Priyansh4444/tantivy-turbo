from pathlib import Path
import subprocess,hashlib,json,re,difflib
base=Path('/home/pronsh/Coding/playground/search')
wt=base/'tantivy-or-capped-expansion-oct10'
held=base/'tantivy-or-metadata-range-oct08'
out=base/'bench/lucene-rs-or-hillclimb-oct07'
sha=lambda data:hashlib.sha256(data).hexdigest()
rels=['src/query/boolean_query/or_maxscore.rs','src/query/term_query/term_scorer.rs']
head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=wt,text=True).strip()
assert head=='43f06833a325d2a18c03bd7e4fa65f2f27944292'
changed=subprocess.check_output(['git','diff','--name-only'],cwd=wt,text=True).splitlines()
assert changed==rels
subprocess.run(['git','diff','--check'],cwd=wt,check=True)
a=(held/rels[0]).read_text(); b=(wt/rels[0]).read_text()
p=subprocess.check_output(['git','show',f'{head}:{rels[0]}'],cwd=wt).decode()
term=(wt/rels[1]).read_bytes(); assert term==(held/rels[1]).read_bytes()
assert b[b.index('    fn check_scheduled_thresholds('):b.index('    fn cached_certificate_survives')]==a[a.index('    fn check_scheduled_thresholds('):a.index('    fn cached_certificate_survives')]
assert b[b.index('    fn single_driver_batches_replay'):]==a[a.index('    fn single_driver_batches_replay'):]
assert b[b.index('        for clause in &mut clauses[first_essential..] {'):b.index('#[cfg(test)]\nmod tests')]==a[a.index('        for clause in &mut clauses[first_essential..] {'):a.index('#[cfg(test)]\nmod tests')]
assert b[b.index('    // Global maxima and preference order are immutable'):b.index('        let mut hi = max_doc;')]==a[a.index('    // Global maxima and preference order are immutable'):a.index('        let mut hi = max_doc;')]
assert b[b.index('    // Global maxima and preference order are immutable'):b.index('        let mut hi = max_doc;')]==p[p.index('    // Global maxima and preference order are immutable'):p.index('        let mut hi = max_doc;')]
assert b[b.index('struct ClauseState {'):b.index('\nimpl ClauseState')]==p[p.index('struct ClauseState {'):p.index('\nimpl ClauseState')]
assert b[b.index('    fn scalar_leaves('):b.index('    fn begin_widening_evidence')]==a[a.index('    fn scalar_leaves('):a.index('    fn begin_widening_evidence')]
assert b[b.index('        let at = |ordinal: usize'):b.index('    #[test]\n    fn wider_region')]==a[a.index('        let at = |ordinal: usize'):a.index('    #[test]\n    fn wider_region')]
# Exact preservation of the indexed original corpus/helpers.
assert b[b.index('    fn indexed_widening_fixture('):b.index('    #[test]\n    fn wider_native_indexed')]==a[a.index('    fn indexed_widening_fixture('):a.index('    #[test]\n    fn wider_native_indexed')]
# Preserve the inherited held13 finite density corpus; only its policy witness changes.
anchor='fn cached_certificate_survives'
before=a[a.index(anchor):a.index('        let (trace, states, choices) =',a.index(anchor))]
after=b[b.index(anchor):b.index('        let leaves = scalar_leaves(&scorers);',b.index(anchor))]
normalize=lambda text: re.sub(r'\s+','',re.sub(r'//[^\n]*','',text))
assert normalize(before)==normalize(after)
# All source13 fixture corpus setup before each first run remains literal.
anchors=[('fn wider_region_restores','            let (accepted, evidence) = check_widening'),('fn wider_single_driver','            let (accepted, evidence) = check_widening'),('fn wider_certificates_cover','                        let (_, evidence) = check_widening'),('fn wider_stale_full','        let (accepted, evidence) = check_widening'),('fn wider_native_indexed','                    let (_, evidence) = check_widening'),('fn wider_native_default_crossed','        let (_, evidence) = check_widening'),('fn wider_legacy_custom','                let (_, evidence) = check_widening'),('fn wider_real_indexed','        let expected = run(false);'),('fn wider_simulated','        let expected = run(false);')]
for anchor,end in anchors:
 before=a[a.index(anchor):a.index(end,a.index(anchor))]
 actualend=end.replace('let (_, evidence) =','let (accepted, evidence) =') if 'native_default_crossed' in anchor else end
 after=b[b.index(anchor):b.index(actualend,b.index(anchor))]
 assert before==after,anchor
# Midpoint corpus/comment changed only to describe the capped policy accurately.
before=a[a.index('fn wider_midpoint'):a.index('        for order in',a.index('fn wider_midpoint'))]
after=b[b.index('fn wider_midpoint'):b.index('        for order in',b.index('fn wider_midpoint'))]
assert re.sub(r'//[^\n]*','',before)==re.sub(r'//[^\n]*','',after)
fullpatch=subprocess.check_output(['git','diff','--',*rels],cwd=wt)
(out/'attempt-16-full.patch').write_bytes(fullpatch)
(out/'attempt-16-source-or.rs').write_bytes((wt/rels[0]).read_bytes())
(out/'attempt-16-source-term.rs').write_bytes(term)
(out/'attempt-16-vs-13.patch').write_text(''.join(difflib.unified_diff(a.splitlines(keepends=True),b.splitlines(keepends=True),fromfile='held13/or_maxscore.rs',tofile='candidate16/or_maxscore.rs')))
receipt={'attempt':16,'status':'source_only_unmeasured','parent':head,'worktree':str(wt),'changed_paths':changed,'source_sha256':{rel:sha((wt/rel).read_bytes()) for rel in rels},'patch_sha256':sha(fullpatch),'held13_term_byte_identical':True,'held13_original_fixture_suffix_preserved_except_one_density_path_witness':True,'held13_replay_byte_identical':True,'accepted11_hot_record_byte_identical':True,'accepted11_global_density_cache_byte_identical':True,'scalar_leaf_cursor_callback_oracles_byte_identical':True,'held13_indexed_original_corpus_byte_identical':True,'all_held13_fixture_corpus_setup_preserved':True,'new_fixture_names':re.findall(r'fn (capped_[a-z0-9_]+)\(',b),'tests_run':False,'builds_run':False,'release_observer_added':False,'additional_logical_scratch_beyond13':0,'selected_end_scratch_logical_bytes':128,'rustfmt_ran':True,'git_diff_check_passed':True}
(out/'attempt-16-static-receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps(receipt,indent=2))
