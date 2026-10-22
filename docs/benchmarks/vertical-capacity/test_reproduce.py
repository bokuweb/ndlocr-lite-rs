import unittest,tempfile,hashlib,json,zipfile,importlib.util
from pathlib import Path
spec=importlib.util.spec_from_file_location('vertical_capacity_reproduce',Path(__file__).with_name('reproduce.py'));module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
def sha(b):return hashlib.sha256(b).hexdigest()
class Restore(unittest.TestCase):
 def setUp(self):self.t=tempfile.TemporaryDirectory();self.addCleanup(self.t.cleanup);self.root=Path(self.t.name);self.packet=self.root/'packet';self.packet.mkdir();self.output=self.root/'restored';self.make()
 def make(self,bad_path=False):
  hashes={}
  for name in ('inputs','evidence'):
   key='../outside' if bad_path and name=='inputs' else ('data/input.txt' if name=='inputs' else 'data/table.csv');raw=b'fixed';aliases={'data/alias.txt':{'source':key,'sha256':sha(raw)}} if name=='inputs' else {};alias=name+'-aliases.json';a=json.dumps(aliases).encode();members={key:sha(raw),alias:sha(a)}
   with zipfile.ZipFile(self.packet/(name+'.zip'),'w')as z:z.writestr(key,raw);z.writestr(alias,a)
   h=sha((self.packet/(name+'.zip')).read_bytes());hashes[name]=h;(self.packet/(name+'.json')).write_text(json.dumps({'sha256':h,'members':members,'alias_member':alias}))
  (self.packet/'archives.json').write_text(json.dumps({'archives':hashes}))
 def test_all_members_and_aliases_are_restored(self):
  module.restore(self.packet,self.output);self.assertEqual((self.output/'data/input.txt').read_bytes(),b'fixed');self.assertEqual((self.output/'data/alias.txt').read_bytes(),b'fixed');self.assertEqual((self.output/'data/table.csv').read_bytes(),b'fixed')
 def test_changed_archive_is_rejected(self):
  p=self.packet/'inputs.zip';p.write_bytes(p.read_bytes()+b'changed')
  with self.assertRaises((ValueError,AssertionError)):module.restore(self.packet,self.output)
 def test_existing_output_is_preserved(self):
  self.output.mkdir();(self.output/'keep').write_text('keep')
  with self.assertRaises(ValueError):module.restore(self.packet,self.output)
  self.assertEqual((self.output/'keep').read_text(),'keep')
 def test_path_traversal_is_rejected(self):
  self.make(True)
  with self.assertRaises((ValueError,AssertionError)):module.restore(self.packet,self.output)
  self.assertFalse((self.root/'outside').exists())
 def test_inputs_only_cannot_claim_full_reproduction(self):
  p=self.packet/'archives.json';x=json.loads(p.read_text());del x['archives']['evidence'];p.write_text(json.dumps(x))
  with self.assertRaises(ValueError):module.restore(self.packet,self.output)
class ReplayPaths(unittest.TestCase):
 def test_only_input_and_callback_paths_are_resolved_without_mutating_frozen_manifest(self):
  original={'inputs':[{'id':'a','rgb':'data/input.rgb','base':{'text':'甲　乙','mean_bits':123},'model_input':[256,24],'chunks':[{'rgb':'parts/one.rgb','text':'甲　乙','mean_bits':456}]}]};copy=json.loads(json.dumps(original));root=Path('/restored').resolve();actual=module.resolved_replay_manifest(original,root);self.assertEqual(actual['inputs'][0]['rgb'],str(root/'data/input.rgb'));self.assertEqual(actual['inputs'][0]['chunks'][0]['rgb'],str(root/'parts/one.rgb'));self.assertEqual(actual['inputs'][0]['base'],original['inputs'][0]['base']);self.assertEqual(actual['inputs'][0]['model_input'],[256,24]);self.assertEqual(original,copy)

if __name__=='__main__':unittest.main()
