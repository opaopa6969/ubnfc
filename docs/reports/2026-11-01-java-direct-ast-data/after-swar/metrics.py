from pathlib import Path
import json,re,statistics as st
root=Path('/tmp/tmp.zkxXajBnzc/after-swar/bench/results')
def java(path):
 rows={}
 for line in path.read_text().splitlines():
  if not line.startswith(('complex,','complex-x64,','atoms-512,')):continue
  a=line.split(',',9)
  rows[(a[0],a[1])]={'median':float(a[5]),'min':float(a[6]),'max':float(a[7]),'bytes':int(a[8])}
 return rows
def rust(path):
 rows={}
 for line in path.read_text().splitlines():
  if not line.startswith(('complex,','complex-x64,')):continue
  a=line.split(',',4); rows[(a[0],a[1])]={'median':float(a[2]),'min':float(a[3]),'max':max(json.loads(a[4]))}
 return rows
def loads(path):
 return [float(x) for x in re.findall(r'# run=\d+ load=([\d.]+)',path.read_text())]
p4=[]; layers=[]; operators=[]
for i in range(1,4):
 jp=java(root/f's{i}-p4.csv'); rs=rust(root/f's{i}-rust.csv')
 for fixture in ['complex','complex-x64']:
  a=jp[(fixture,'ast-direct')]; b=jp[(fixture,'ast-recipe')]
  ar=jp[(fixture,'recognize-direct')];br=jp[(fixture,'recognize-recipe')]
  p4.append({'session':i,'fixture':fixture,'direct':a,'recipe':b,'recognize_direct':ar,'recognize_recipe':br,'ast_ratio':a['median']/b['median'],'control_ratio':ar['median']/br['median'],'load_min':min(loads(root/f's{i}-p4.csv')),'load_max':max(loads(root/f's{i}-p4.csv'))})
  jl=a['median']-ar['median']; rl=rs[(fixture,'ast')]['median']-rs[(fixture,'recognize')]['median']
  layers.append({'session':i,'fixture':fixture,'java_layer':jl,'rust_layer':rl,'ratio':jl/rl,'rust_ast':rs[(fixture,'ast')],'rust_recognize':rs[(fixture,'recognize')],'rust_load_min':min(loads(root/f's{i}-rust.csv')),'rust_load_max':max(loads(root/f's{i}-rust.csv'))})
for i in range(1,4):
 path=root/f's{i}-operators.csv'; rows=java(path)
 if ('atoms-512','ast-direct') not in rows:continue
 a=rows[('atoms-512','ast-direct')]; b=rows[('atoms-512','ast-recipe')];h=rows[('atoms-512','ast-handwritten')]
 ar=rows[('atoms-512','recognize-direct')];br=rows[('atoms-512','recognize-recipe')]
 operators.append({'session':i,'direct':a,'recipe':b,'handwritten':h,'ratio':a['median']/h['median'],'control_ratio':ar['median']/br['median'],'load_min':min(loads(path)),'load_max':max(loads(path))})
representative={}
for fixture in ['complex','complex-x64']:
 cells=[r for r in p4 if r['fixture']==fixture];ls=[r for r in layers if r['fixture']==fixture]
 jl=st.median(r['java_layer'] for r in ls);rl=st.median(r['rust_layer'] for r in ls)
 representative[fixture]={'ast':st.median(r['direct']['median'] for r in cells),'recipe_ast':st.median(r['recipe']['median'] for r in cells),'recognize':st.median(r['recognize_direct']['median'] for r in cells),'java_layer':jl,'rust_layer':rl,'layer_ratio':jl/rl}
if len(operators)==3:
 representative['operators']={'ast':st.median(r['direct']['median'] for r in operators),'recipe_ast':st.median(r['recipe']['median'] for r in operators),'handwritten':st.median(r['handwritten']['median'] for r in operators)}
summary={'p4':p4,'layers':layers,'operators':operators,'representative':representative}
(root/'metrics.json').write_text(json.dumps(summary,ensure_ascii=False,indent=2)+'\n')
print(json.dumps(summary,ensure_ascii=False,indent=2))
