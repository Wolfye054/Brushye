#!/usr/bin/env python3
"""L-008 run 1: compare CSP mask exports (obs dir) with Brushye's alpha model.
usage: l008_check.py OBS_DIR   (needs target/release/brushye)"""
import sys, subprocess, base64, numpy as np
from PIL import Image
R='/home/geoffrey/repos/brushye'; O=sys.argv[1]; brushye=R+'/target/release/brushye'
I=lambda p: np.array(Image.open(p).convert('RGBA'))
base=I(R+'/specs/behavior/blending/data/inputs/base.png'); top=I(R+'/specs/behavior/blending/data/inputs/top.png')
alone=I(O+'/top_alone.png'); M=alone[...,3]
def cmp(name, a, b):
    d=np.abs(a.astype(int)-b.astype(int)); n=(d.max(-1)>0).sum()
    print(f'{name:38s} pixels differing {n:6d} / {a.shape[0]*a.shape[1]}  max {d.max()}')
    return d
cmp('applied_top_alone == top_alone', I(O+'/applied_top_alone.png'), alone)
cmp('applied_composite == composite_normal', I(O+'/applied_composite.png'), I(O+'/composite_normal.png'))
cmp('mask disabled == top (opaque Normal)', I(O+'/composite_mask_disabled.png'), top)
def render(blend, opacity):
    t=top.copy(); t[...,3]=M
    b64=lambda a: 'b64:'+base64.b64encode(a.tobytes()).decode()
    s=f"""doc.new 256 512 paper=none
layer.new L1
layer.pixels L1 x=0 y=0 w=256 h=512 data={b64(base)}
layer.new L2
layer.pixels L2 x=0 y=0 w=256 h=512 data={b64(t)}
layer.set L2 blend={blend} opacity={opacity}
"""
    out=f'{O}/pred_{blend}_{opacity}.png'
    subprocess.run([brushye,'run','-','--png',out],input=s.encode(),check=True)
    return I(out)
for blend,op,f in [('normal',100,'composite_normal'),('multiply',100,'composite_multiply'),('normal',50,'composite_op50')]:
    obs=I(f'{O}/{f}.png'); d=cmp(f'{f} vs alpha=M model', obs, render(blend,op))
    if d.max():
        bad=np.argwhere(d.max(-1)>0)
        for y,x in bad[:4]: print('   ',y,x,'M',M[y,x],'base',base[y,x,:3],'top',top[y,x,:3],'obs',obs[y,x],'pred',render.__name__)
