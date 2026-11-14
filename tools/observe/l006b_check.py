#!/usr/bin/env python3
"""l006b: compare CSP exports in obs-out/m10/l006b with Brushye-rendered predictions (needs target/release/brushye)."""
import subprocess, base64, numpy as np
from PIL import Image
R='/home/geoffrey/repos/brushye'; O=R+'/obs-out/m10/l006b'; B=R+'/target/release/brushye'
I=lambda p: np.array(Image.open(p).convert('RGBA')).astype(np.int64)
inp=R+'/specs/behavior/blending/data/inputs/'
base=I(inp+'base.png'); top=I(inp+'top.png'); mid=I(inp+'top_alpha.png')
def render(layers,name):
    s='doc.new 256 512 paper=none\n'
    for i,(px,blend,op) in enumerate(layers,1):
        s+=f'layer.new L{i}\nlayer.pixels L{i} x=0 y=0 w=256 h=512 data=b64:{base64.b64encode(px.astype(np.uint8).tobytes()).decode()}\nlayer.set L{i} blend={blend} opacity={op}\n'
    out=f'{O}/pred_{name}.png'; subprocess.run([B,'run','-','--png',out],input=s.encode(),check=True); return I(out)
def cmp(name,obs,pred):
    d=np.abs(obs-pred); print(f'{name:46s} differing px {(d.max(-1)>0).sum():6d}  max {d.max()}')
def group(clip_blend, clip_op, base_blend, name):
    m=mid.copy(); m[...,3]=255
    g=render([(m,'normal',100),(top,clip_blend,clip_op)],name+'_g'); g[...,3]=mid[...,3]
    return render([(base,'normal',100),(g,base_blend,100)],name)
cmp('clip Normal (L-006 model)', I(O+'/clip_normal.png'), group('normal',100,'normal','n'))
cmp('clipped layer at 50 %', I(O+'/clip_clipped_op50.png'), group('normal',50,'normal','op50'))
cmp('clip base in Multiply', I(O+'/clip_base_multiply.png'), group('normal',100,'multiply','bm'))
cmp('clip onto Normal folder (= clip Normal)', I(O+'/clip_onto_folder_normal.png'), I(O+'/clip_normal.png'))
cmp('clip onto Through folder vs clip Normal', I(O+'/clip_onto_folder_through.png'), I(O+'/clip_normal.png'))
cmp('clip onto Through folder vs unclipped', I(O+'/clip_onto_folder_through.png'), render([(base,'normal',100),(mid,'normal',100),(top,'normal',100)],'unclipped'))
