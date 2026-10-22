import unittest,json,zipfile
from pathlib import Path
class TableCoverage(unittest.TestCase):
 def test_all_formal_csv_and_markdown_are_restorable(self):
  r=Path('tmp/real-vertical-scan');packet=Path('docs/benchmarks/real-vertical-scan');keys=set()
  for name in json.loads((packet/'archives.json').read_text())['archives']:
   index=json.loads((packet/(name+'.json')).read_text());keys.update(index['members'])
   with zipfile.ZipFile(packet/(name+'.zip'))as z:keys.update(json.loads(z.read(name+'-aliases.json')))
  expected={str(p)for folder in ['initial-scores','line-scores']for p in(r/folder).iterdir()if p.suffix in ['.csv','.md']};self.assertEqual(len(expected),12);self.assertEqual(expected-keys,set())
if __name__=='__main__':unittest.main()
