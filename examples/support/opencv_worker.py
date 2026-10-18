"""本流と同じ uint8 INTER_LINEAR を使う、診断だけの固定 oracle。"""
import struct
import sys
import cv2
import numpy as np


def read_exact(stream, size):
    """pipe の短い read を結合し、途中 EOF を拒否する。"""
    chunks = bytearray()
    while len(chunks) < size:
        block = stream.read(size - len(chunks))
        if not block:
            raise EOFError('truncated oracle input')
        chunks.extend(block)
    return bytes(chunks)


def preprocess(rgb, width, height, input_width, input_height):
    """横書き固定の同一 RGB を本流の順番で正規化する。"""
    image = np.frombuffer(rgb, dtype=np.uint8).reshape(height, width, 3)
    resized = cv2.resize(image, (input_width, input_height), interpolation=cv2.INTER_LINEAR)
    tensor = np.ascontiguousarray(resized[:, :, ::-1]).astype(np.float32)
    tensor /= 127.5
    tensor -= 1.0
    return tensor.transpose(2, 0, 1).astype('<f4').tobytes()


def main():
    """版を固定し、正解や OCR 出力を読まず RGB だけを処理する。"""
    if cv2.__version__ != '4.12.0' or np.__version__ != '2.2.6':
        raise RuntimeError('requires OpenCV 4.12.0 and NumPy 2.2.6')
    cv2.setNumThreads(1)
    source, output = sys.stdin.buffer, sys.stdout.buffer
    while True:
        first = source.read(1)
        if not first:
            return
        width, height, iw, ih = struct.unpack('<4I', first + read_exact(source, 15))
        if min(width, height, iw, ih) == 0:
            raise ValueError('zero dimension')
        tensor = preprocess(read_exact(source, width * height * 3), width, height, iw, ih)
        output.write(struct.pack('<Q', len(tensor)))
        output.write(tensor)
        output.flush()


if __name__ == '__main__':
    main()
