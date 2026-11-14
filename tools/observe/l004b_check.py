#!/usr/bin/env python3
"""l004b: compare CSP exports in obs-out/m10/l004b with Brushye-rendered predictions (needs target/release/brushye)."""
import sys, subprocess, base64, numpy as np
from PIL import Image
R='/home/geoffrey/repos/brushye'; O=R+'/obs-out/m10/l004b'; B=R+'/target/release/brushye'
I=lambda p: np.array(Image.open(p).convert('RGBA')).astype(np.int64)
inp=R+'/specs/behavior/blending/data/inputs/'
base=I(inp+'base.png'); top=I(inp+'top.png'); ta=I(inp+'top_alpha.png')
M=I(R+'/specs/behavior/blending/data/observed3/mask2_readout_alone.png')[...,3]
def render(layers, paper='none', name='x'):
    s=f'doc.new 256 512 paper={paper}\n'
    for i,(px,blend,op) in enumerate(layers,1):
        s+=f'layer.new L{i}\nlayer.pixels L{i} x=0 y=0 w=256 h=512 data=b64:{base64.b64encode(px.astype(np.uint8).tobytes()).decode()}\nlayer.set L{i} blend={blend} opacity={op}\n'
    out=f'{O}/pred_{name}.png'; subprocess.run([B,'run','-','--png',out],input=s.encode(),check=True); return I(out)
def cmp(name,obs,pred):
    d=np.abs(obs-pred); print(f'{name:44s} differing px {(d.max(-1)>0).sum():6d}  max {d.max()}')
a15=lambda x:(x*32768+127)//255; lerp=lambda x,y,w:(x*(32768-w)+y*w+16384)>>15
through=render([(base,'normal',100),(top,'multiply',100),(ta,'screen',100)],name='through')
cmp('Through (Multiply + Screen children)', I(O+'/folder_through.png'), through)
iso=render([(top,'multiply',100),(ta,'screen',100)],name='iso')
normal=render([(base,'normal',100),(iso,'normal',100)],name='normal')
cmp('Normal folder (isolated)', I(O+'/folder_normal.png'), normal)
cmp('Through, child top_alpha at 50 %', I(O+'/folder_through_child_op50.png'), render([(base,'normal',100),(top,'multiply',100),(ta,'screen',50)],name='through50'))
for nm,res in [('Through',through),('Normal',normal)]:
    obs=I(O+f'/folder_{nm.lower()}_mask.png')
    cmp(f'{nm} + mask: lerp(base, result, a15(M))', obs[...,:3], lerp(base[...,:3],res[...,:3],a15(M)[...,None]))
    isoM=iso.copy(); isoM[...,3]=iso[...,3]*M//255
    if nm=='Normal': cmp('Normal + mask: iso alpha floor(a*M/255) over base', obs, render([(base,'normal',100),(isoM,'normal',100)],name='normal_mask'))
print('--- Through + mask variants')
obs=I(O+'/folder_through_mask.png')
tm=top.copy(); tm[...,3]=255*M//255
tam=ta.copy(); tam[...,3]=ta[...,3]*M//255
cmp('mask on each child (floor a*M/255)', obs, render([(base,'normal',100),(tm,'multiply',100),(tam,'screen',100)],name='tm_child'))
for nm,w in [('w=(M*32768)//255',(M*32768)//255),('w=M<<7',M<<7),('w=(M*32768+128)//255',(M*32768+128)//255),('w=M*128+M//2',M*128+M//2)]:
    cmp('lerp(base,through,'+nm+')', obs[...,:3], lerp(base[...,:3],through[...,:3],w[...,None]))
d=np.abs(obs[...,:3]-lerp(base[...,:3],through[...,:3],a15(M)[...,None])).max(-1)
for y,x in np.argwhere(d>0)[:6]: print('  ',y,x,'M',M[y,x],'base',base[y,x,:3],'thr',through[y,x,:3],'obs',obs[y,x,:3])
