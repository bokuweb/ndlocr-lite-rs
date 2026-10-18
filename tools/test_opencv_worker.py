"""固定画素 oracle と実プロセスの通信を独立に照合する。"""
import json
from pathlib import Path
import struct
import subprocess
import sys
import unittest
import numpy as np

ROOT = Path(__file__).resolve().parents[1]
WORKER = ROOT / 'examples/support/opencv_worker.py'


class WorkerTests(unittest.TestCase):
    """画像認識を行わず画素値・色順・配置・EOFを検査する。"""

    def test_all_pixels_and_multiple_requests(self):
        """恒等・分数・拡大・1画素・縮小を固定byteと全件比較する。"""
        fixture = json.loads((ROOT / 'tests/fixtures/opencv_resize/pixels.json').read_text())
        request = b''.join(struct.pack('<4I', *c['dimensions']) + bytes(c['rgb']) for c in fixture['cases'])
        result = subprocess.run([sys.executable, '-u', str(WORKER)], input=request, capture_output=True, check=True)
        offset = 0
        for case in fixture['cases']:
            _,_,iw,ih = case['dimensions']
            rgb = np.array(case['resized_rgb'], dtype=np.uint8).reshape(ih,iw,3)
            # 生成器やworkerをimportせず、保存した整数画素から期待tensorを構成する。
            expected = (rgb.astype(np.float32) / np.float32(127.5) - np.float32(1)).transpose(2,0,1)[::-1]
            size, = struct.unpack_from('<Q', result.stdout, offset)
            self.assertEqual(size, iw*ih*3*4)
            offset += 8
            self.assertEqual(result.stdout[offset:offset+size], expected.astype('<f4').tobytes(), case['id'])
            offset += size
        self.assertEqual(offset, len(result.stdout))

    def test_truncated_and_zero_input_fail(self):
        """途中header・途中RGB・0寸法で成功応答を出さない。"""
        for request in [b'\x01', struct.pack('<4I',1,1,1,1)+b'\x00', struct.pack('<4I',0,1,1,1)]:
            result = subprocess.run([sys.executable, '-u', str(WORKER)], input=request, capture_output=True)
            self.assertNotEqual(result.returncode,0)
            self.assertEqual(result.stdout,b'')


if __name__ == '__main__':
    unittest.main()
