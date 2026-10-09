"""Generate platform icon representations from the padded 1024px master (Pillow)."""
from pathlib import Path
from PIL import Image
from io import BytesIO
import struct

root = Path(__file__).resolve().parents[1]
master = Image.open(root / 'assets/suiza-icon.png').convert('RGBA')
assert master.size == (1024, 1024)
for size in (16, 24, 32, 48, 64, 128, 256, 512, 1024):
    folder = root / 'assets/icons' / str(size)
    folder.mkdir(parents=True, exist_ok=True)
    master.resize((size, size), Image.Resampling.LANCZOS).save(folder / 'suiza.png')
master.save(root / 'assets/suiza.ico', sizes=[(n,n) for n in (16,24,32,48,64,128,256)])
master.resize((512,512), Image.Resampling.LANCZOS).save(root / 'assets/suiza-icon-preview.png')
# PNG-backed ICNS representations, including 16px at 1x and all Retina sizes.
blocks=[]
for tag, size in [(b'icp4',16),(b'icp5',32),(b'icp6',64),(b'ic07',128),(b'ic08',256),(b'ic09',512),(b'ic10',1024),(b'ic11',32),(b'ic12',64),(b'ic13',256),(b'ic14',512)]:
    data=BytesIO()
    master.resize((size,size),Image.Resampling.LANCZOS).save(data,format='PNG')
    payload=data.getvalue()
    blocks.append(tag+struct.pack('>I',len(payload)+8)+payload)
body=b''.join(blocks)
(root / 'assets/suiza.icns').write_bytes(b'icns'+struct.pack('>I',len(body)+8)+body)
