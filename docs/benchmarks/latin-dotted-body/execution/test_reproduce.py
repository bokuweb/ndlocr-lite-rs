import unittest,tempfile,zipfile,json,hashlib
from pathlib import Path
import reproduce
sha=lambda b:hashlib.sha256(b).hexdigest()
class RestoreTests(unittest.TestCase):
 def build(self,p,files,aliases=None):
  files=files|{'part-0-aliases.json':(json.dumps(aliases or {})+'\n').encode()}
  with zipfile.ZipFile(p/'part-0.zip','w')as z:
   for k,v in files.items():z.writestr(k,v)
  index={'sha256':sha((p/'part-0.zip').read_bytes()),'members':{k:sha(v)for k,v in files.items()}}
  (p/'part-0.json').write_text(json.dumps(index));(p/'archives.json').write_text(json.dumps({'archives':{'part-0':index['sha256']}}))
 def test_round_trip_preserves_utf8_bytes_and_alias(self):
  with tempfile.TemporaryDirectory()as t:
   p=Path(t)/'packet';p.mkdir();out=Path(t)/'out';b=' ）ij) \n'.encode();self.build(p,{'data/a.txt':b},{'data/b.txt':{'source':'data/a.txt','sha256':sha(b)}});reproduce.restore(p,out);self.assertEqual((out/'data/a.txt').read_bytes(),b);self.assertEqual((out/'data/b.txt').read_bytes(),b)
 def test_rejects_changed_archive_and_member(self):
  for archive in [True,False]:
   with self.subTest(archive=archive),tempfile.TemporaryDirectory()as t:
    p=Path(t)/'packet';p.mkdir();self.build(p,{'data/a':b'a'});out=Path(t)/'out'
    if archive:(p/'part-0.zip').write_bytes((p/'part-0.zip').read_bytes()+b'x')
    else:
     d=json.loads((p/'part-0.json').read_text());d['members']['data/a']='0'*64;(p/'part-0.json').write_text(json.dumps(d))
    with self.assertRaises((AssertionError,ValueError)):reproduce.restore(p,out)
 def test_rejects_escaping_member_and_alias(self):
  for alias in [True,False]:
   with self.subTest(alias=alias),tempfile.TemporaryDirectory()as t:
    p=Path(t)/'packet';p.mkdir();self.build(p,{'data/a':b'a'} if alias else {'../escape':b'a'},{'../escape':{'source':'data/a','sha256':sha(b'a')}}if alias else None)
    with self.assertRaises((AssertionError,ValueError)):reproduce.restore(p,Path(t)/'out')
    self.assertFalse((Path(t)/'escape').exists())
 def test_rejects_wrong_alias_hash_and_nonempty_destination(self):
  with tempfile.TemporaryDirectory()as t:
   p=Path(t)/'packet';p.mkdir();self.build(p,{'data/a':b'a'},{'data/b':{'source':'data/a','sha256':'0'*64}})
   with self.assertRaises((AssertionError,ValueError)):reproduce.restore(p,Path(t)/'out')
   out=Path(t)/'existing';out.mkdir();(out/'keep').write_bytes(b'keep')
   with self.assertRaises((AssertionError,ValueError)):reproduce.restore(p,out)
   self.assertEqual((out/'keep').read_bytes(),b'keep')
if __name__=='__main__':unittest.main()
