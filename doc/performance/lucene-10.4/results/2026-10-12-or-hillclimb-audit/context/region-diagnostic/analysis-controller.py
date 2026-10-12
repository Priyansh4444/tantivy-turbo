import json,hashlib,shutil
from pathlib import Path
folder=Path('/home/pronsh/Coding/playground/search/bench/lucene-rs-or-hillclimb-oct07/region-diagnostic')
records=json.loads((folder/'run/records.json').read_text())
rows=[];without=[];totals={}
for record in records:
    lines=record['diagnostic_lines']
    stats=[json.loads(line.removeprefix('OR_REGION_DIAGNOSTIC ')) for line in lines if line.startswith('OR_REGION_DIAGNOSTIC ')]
    assert len(stats)<=1,(record['case']['id'],lines)
    if not stats:
        without.append(record['case']);continue
    s=stats[0]
    assert not s['fallback']
    assert sum(s['physical_widths'])==sum(s['essential_counts'])==sum(s['cap_source_masks'])==s['regions']
    assert s['skipped_regions']==s['essential_counts'][0]
    assert sum(s['matching_essentials'])==s['scalar_candidates']==s['scalar_boundaries']
    assert s['gap_ge_2048']<=s['gap_ge_128']<=s['unique_earliest']<=s['scalar_boundaries']
    assert s['optional_seeks']<=s['optional_probes']<=s['optional_bound_checks']
    rows.append({'case':record['case'],'stats':s})
    for key,value in s.items():
        if key in ('version','num_terms','max_doc','fallback'):continue
        if isinstance(value,list):
            totals.setdefault(key,[0]*len(value))
            totals[key]=[a+b for a,b in zip(totals[key],value)]
        else:totals[key]=totals.get(key,0)+value
regions=totals['regions'];batches=totals['batch_lengths']
summary={'diagnostic_only':True,'queries':len(records),'or_kernel_invocations':len(rows),'queries_without_kernel':without,
    'totals':totals,
    'fractions':{
        'regions_width_below128':sum(totals['physical_widths'][:8])/regions,
        'regions_width_below2048':sum(totals['physical_widths'][:12])/regions,
        'regions_capped_only_locally_optional':totals['cap_source_masks'][1]/regions,
        'regions_capped_optional_including_ties':sum(n for mask,n in enumerate(totals['cap_source_masks']) if mask&1)/regions,
        'regions_capped_essential_including_ties':sum(n for mask,n in enumerate(totals['cap_source_masks']) if mask&2)/regions,
        'regions_skipped':totals['skipped_regions']/regions,
        'scalar_candidates_unique_earliest':totals['unique_earliest']/totals['scalar_candidates'],
        'scalar_boundaries_capped_gap128':totals['gap_ge_128']/totals['scalar_candidates'],
        'scalar_boundaries_capped_gap2048':totals['gap_ge_2048']/totals['scalar_candidates'],
        'nonempty_batch_mean_postings':sum(i*n for i,n in enumerate(batches))/sum(batches[1:]),
        'nonempty_batch_len_at_most8':sum(batches[1:9])/sum(batches[1:])},
    'largest_region_queries':sorted(rows,key=lambda r:r['stats']['regions'],reverse=True)[:10],
    'limits':'Counters alter runtime; no latency or causal speed evidence. Counts unweighted by query latency. Address gaps are overlapping scalar boundary opportunities, not realized batch spans. Kernel-free queries use other routes. No bound-policy provenance or hypothetical range-scan cost.'}
(folder/'analysis.json').write_text(json.dumps(summary,indent=2)+'\n')
shutil.copy2(__file__,folder/'analysis-controller.py')
print(json.dumps({k:v for k,v in summary.items() if k not in ('totals','largest_region_queries','queries_without_kernel')},indent=2))
print(json.dumps({'totals':{k:v for k,v in totals.items() if not isinstance(v,list)},'essential_counts':totals['essential_counts'],'cap_source_masks':totals['cap_source_masks'],'largest_queries':[(r['case']['id'],r['stats']['regions']) for r in summary['largest_region_queries']]}))
