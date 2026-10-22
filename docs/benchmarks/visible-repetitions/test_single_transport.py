import unittest
from single_transport import recognition_content
class SingleTransportTests(unittest.TestCase):
 def test_removes_only_one_println_terminator(self):
  for payload in [b'',b'000000',' 日本語😀 '.encode('utf-8'),b'  x\t ',b'x\r',b'\0x']:
   self.assertEqual(recognition_content(payload+b'\n'),payload)
 def test_missing_or_multiple_newlines_are_errors(self):
  for raw in [b'',b'123',b'123\n\n',b'123\n456\n']:
   with self.assertRaises(ValueError):recognition_content(raw)
if __name__=='__main__':unittest.main()
