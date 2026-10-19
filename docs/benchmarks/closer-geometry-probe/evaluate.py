from pathlib import Path
import json,os,csv,subprocess,collections
r=Path(os.environ.get('NDLOCR_GEOMETRY_EVAL_ROOT','tmp/closer-geometry'));m=json.loads((r/'evaluation-manifest.json').read_text(encoding='utf-8'));d=json.loads((r/'probe.json').read_text(encoding='utf-8'))if(r/'probe.json').exists()else {'models':[{'label':'single'+str(n),'rows':[dict(id=x['id'],raw_source=x['raw_source'],truth=(r/'truth'/(x['id']+'.txt')).read_text(encoding='utf-8'),**{key:(r/'policies'/key/('single'+str(n))/(x['id']+'.txt')).read_text(encoding='utf-8')for key in ['baseline','raw','fragment','no_space','short_common','closer_body']})for x in m['inputs']]}for n in [30,50,100]]};inputs={x['id']:x for x in m['inputs']};names=['baseline','raw','fragment','no_space','short_common','closer_body'];stats={};regressions=[];raw_stats={}
for model in d['models']:
 label=model['label'];rows=model['rows'];assert len(rows)==1239
 raw_stats[label]={'raw_exact_lost_before':sum(x['raw']==x['truth']and x['baseline']!=x['truth']for x in rows),'old80_lost':sum(x['raw_source']=='reused_raw'and x['raw']==x['truth']and x['baseline']!=x['truth']for x in rows)}
 for x in rows:
  for name in names:
   p=r/'policies'/name/label/(x['id']+'.txt');p.parent.mkdir(parents=True,exist_ok=True);p.write_text(x[name],encoding='utf-8')
 s={name:collections.Counter()for name in names if name!='baseline'}
 for norm in ['none','width','strict']:
  out=r/'metrics';out.mkdir(exist_ok=True);stem=label+'-'+norm;argv=['python3','tools/eval_ocr_metrics.py','--truth-dir',str(r/'truth')]
  for name in names:argv+=['--system',name+'='+str(r/'policies'/name/label)]
  argv+=['--normalize',norm,'--require-all','--output-csv',str(out/(stem+'.csv')),'--output-md',str(out/(stem+'.md'))];subprocess.run(argv,check=True,capture_output=True)
  for ext in ['.csv','.md']:
   p=out/(stem+ext);p.write_bytes(p.read_bytes().replace(b'\r\n',b'\n').rstrip()+b'\n')
  report=list(csv.DictReader((out/(stem+'.csv')).open()));baseline={x['file']:x for x in report if x['system']=='baseline'};assert len(baseline)==1239
  for x in report:
   name=x['system']
   if name=='baseline':continue
   before=baseline[x['file']];old,new=int(before['char_errors']),int(x['char_errors']);bword,aword=int(before['word_errors']),int(x['word_errors']);s[name]['char_improved'if new<old else'char_regressed'if new>old else'char_equal']+=1;s[name]['word_regressed']+=aword>bword;id=Path(x['file']).stem
   if new>old or aword>bword:regressions.append({'model':label,'policy':name,'normalize':norm,'id':id,'cohort':inputs[id]['cohort'],'before_char':old,'after_char':new,'before_word':bword,'after_word':aword})
 for name in s:
  for x in rows:
   baseline=x['baseline'];candidate=x[name];t=x['truth'];s[name]['changed']+=baseline!=candidate
   if x['raw']==t and baseline!=t:
    s[name]['raw_exact_restored']+=candidate==t;s[name]['raw_exact_remaining']+=candidate!=t
    if x['raw_source']=='reused_raw':s[name]['old80_restored']+=candidate==t
   if candidate==t and baseline!=t:s[name]['complete_restored']+=1
 stats[label]={name:dict(v)for name,v in s.items()}
assert sum(x['old80_lost']for x in raw_stats.values())==80
out={'comparisons_per_policy':11151,'raw_stats':raw_stats,'stats':stats,'regressions':regressions};(r/'audit.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps({k:out[k]for k in ['comparisons_per_policy','raw_stats','stats']},ensure_ascii=False,indent=2));print('first regressions',json.dumps(regressions[:8],ensure_ascii=False,indent=2))
