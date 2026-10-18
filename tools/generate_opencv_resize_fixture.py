"""独立した cv2 呼出しから小さい uint8 画素 oracle を生成する。"""
import json
from pathlib import Path
import cv2
import numpy as np

assert cv2.__version__ == '4.12.0' and np.__version__ == '2.2.6'
cv2.setNumThreads(1)
cases = []
for name,w,h,iw,ih in [('identity',3,2,3,2), ('fractional',5,3,4,2),
                        ('enlarge',2,3,5,7), ('single-output',2,2,1,1),
                        ('single-source',1,1,4,3), ('half-size',6,4,3,2)]:
    image = ((np.arange(w*h*3, dtype=np.uint32)*73+19) % 256).astype(np.uint8).reshape(h,w,3)
    resized = cv2.resize(image, (iw,ih), interpolation=cv2.INTER_LINEAR)
    cases.append(dict(id=name, dimensions=[w,h,iw,ih], rgb=image.reshape(-1).tolist(),
                      resized_rgb=resized.reshape(-1).tolist()))
path = Path(__file__).resolve().parents[1] / 'tests/fixtures/opencv_resize/pixels.json'
with path.open('x') as output:
    json.dump(dict(opencv=cv2.__version__, numpy=np.__version__, cases=cases), output, indent=2)
    output.write('\n')
