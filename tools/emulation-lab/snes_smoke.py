#!/usr/bin/env python3
"""Exercise real Android SNES launch/save/eject/resume using original test ROMs.

Uses only the isolated debuggable package. Grant the ROM directory in Slot's
Library once before running; no ROM permissions or app data are fabricated.
"""
import argparse
import hashlib
import io
import json
from pathlib import Path
import shlex
import subprocess
import sys
import time
from urllib.parse import unquote
import zipfile

from make_snes_probe import rom

PACKAGE = 'fyi.slot.konkr.diag'
ACTIVITY = PACKAGE + '/fyi.slot.konkr.MainActivity'
PREFIX = 'Slot SNES Smoke '


def fixtures():
    ntsc, pal, hires = bytes(rom()), bytes(rom(pal=True)), bytes(rom(hires=True))
    zipped = io.BytesIO()
    with zipfile.ZipFile(zipped, 'w', zipfile.ZIP_DEFLATED) as archive:
        archive.writestr('Probe.sfc', ntsc)
    return {
        PREFIX + 'NTSC.sfc': ntsc,
        PREFIX + 'PAL.sfc': pal,
        PREFIX + 'Hires.sfc': hires,
        PREFIX + 'Header.smc': bytes(512) + ntsc,
        PREFIX + 'ZIP.zip': zipped.getvalue(),
    }


def probe_visible(png):
    from PIL import Image
    image = Image.open(io.BytesIO(png)).convert('RGB')
    w, h = image.size
    pixels = image.crop((w//3,h//3,w*2//3,h*2//3)).resize((32,32)).tobytes()
    return sum(pixels[i] > 180 and pixels[i+1] < 40 and pixels[i+2] < 40
               for i in range(0,len(pixels),3)) > 950


class Device:
    def __init__(self, serial):
        self.prefix = ['adb', '-s', serial]

    def adb(self, *args, check=True, timeout=30):
        result = subprocess.run(self.prefix + list(args), capture_output=True, timeout=timeout)
        if check and result.returncode:
            raise RuntimeError(result.stderr.decode(errors='replace') or result.stdout.decode(errors='replace'))
        return result.stdout

    def shell(self, *args, **options):
        return self.adb('shell', shlex.join(args), **options)

    def private(self, *args, **options):
        return self.shell('run-as',PACKAGE,*args,**options)

    def json(self, name):
        return json.loads(self.private('cat','files/'+name))

    def key(self, key):
        self.shell('input','keyevent',str(key)); time.sleep(.2)

    def picture(self):
        return self.adb('exec-out','screencap','-p')

    def log(self):
        return self.adb('logcat','-d','-s','SlotKONKR:I','AndroidRuntime:E','*:S').decode(errors='replace')

    def states(self):
        names=self.private('find','files/Saves','-maxdepth','1','-name','*.snes9x.state').decode().splitlines()
        return {name:hashlib.sha256(self.private('cat',name)).hexdigest() for name in names}


def wait_for(check, description, seconds=25):
    deadline=time.monotonic()+seconds
    while time.monotonic()<deadline:
        if check(): return
        time.sleep(.3)
    raise RuntimeError('Timed out: '+description)


def select(device, uri, total):
    for _ in range(5):
        if device.json('konkr-shelf-focus.json').get('rack') == 3: break
        previous=device.json('konkr-shelf-focus.json')
        device.key(103)
        wait_for(lambda:device.json('konkr-shelf-focus.json') != previous,'shelf focus persistence',4)
    else: raise RuntimeError('SNES shelf is not available.')
    for _ in range(total+1):
        if device.json('konkr-shelf-focus.json').get('uri') == uri: return
        previous=device.json('konkr-shelf-focus.json')
        device.key(22)
        wait_for(lambda:device.json('konkr-shelf-focus.json') != previous,'cartridge focus persistence',4)
    raise RuntimeError('Could not select the generated test cartridge.')


def run(args):
    from PIL import Image  # Fail before touching the device if Pillow is absent.
    device=Device(args.serial)
    if device.shell('getprop','ro.product.cpu.abi').strip() != b'arm64-v8a':
        raise RuntimeError('Use an ARM64 emulator or device with the debug APK.')
    device.private('pwd')
    output=Path(args.output); output.mkdir(parents=True,exist_ok=True)
    files=fixtures()
    device.shell('mkdir','-p',args.rom_dir)
    for name, content in files.items():
        target=args.rom_dir.rstrip('/')+'/'+name
        existing=device.shell('cat',target,check=False)
        # ZIP timestamps differ between runs; compare the embedded ROM instead.
        same=existing==content
        if existing and name.endswith('.zip'):
            try:
                with zipfile.ZipFile(io.BytesIO(existing)) as old:
                    same=old.read('Probe.sfc')==bytes(rom())
            except (zipfile.BadZipFile,KeyError): pass
        if existing and not same: raise RuntimeError('Refusing to overwrite a different file: '+target)
        local=output/name; local.write_bytes(content)
        if not same: device.adb('push',str(local),target)
    if args.prepare_only:
        print('Probes prepared. Choose '+args.rom_dir+' in Slot → Library → Choose ROM Folder.'); return
    try: library=device.json('rom-library-cache-v1.json')
    except (RuntimeError,ValueError):
        raise RuntimeError('First choose '+args.rom_dir+' in Slot → Library → Choose ROM Folder, then rerun.')
    expected=args.rom_dir.removeprefix('/sdcard/').removeprefix('/storage/emulated/0/')
    root=unquote(library.get('root','')).split('/tree/')[-1]
    if root != 'primary:'+expected:
        raise RuntimeError('Select '+args.rom_dir+' as the ROM folder first; the current library is untouched.')
    # Relaunch only the isolated test APK; the retail app is never stopped.
    device.shell('am','force-stop',PACKAGE)
    device.shell('am','start','-W','-n',ACTIVITY)
    time.sleep(2)
    for key in [108,96,19,96]: device.key(key)  # Library → Refresh (last row).
    titles={Path(name).stem for name in files}
    wait_for(lambda: titles <= {g['title'] for g in device.json('rom-library-cache-v1.json')['games']},'probe discovery')
    time.sleep(1)
    games=[g for g in device.json('rom-library-cache-v1.json')['games'] if g['platform']=='SNES']
    report={'device':args.serial,'package':PACKAGE,'cases':[]}
    for title in sorted(titles):
        game=next(g for g in games if g['title']==title)
        select(device,game['uri'],len(games))
        before=device.states()
        device.key(96)
        wait_for(lambda:probe_visible(device.picture()),title+' launch')
        (output/(title+'-playing.png')).write_bytes(device.picture())
        time.sleep(1)
        # Slot cap → last pause row (Save & Eject) → A.
        for key in [110,19,96]: device.key(key)
        wait_for(lambda:not probe_visible(device.picture()),title+' eject')
        def saved():
            current=device.states()
            return any(before.get(name)!=digest for name,digest in current.items())
        wait_for(saved,title+' automatic state write')
        battery=device.private('cat','files/Saves/Snes9x/'+title+'.srm')
        if len(battery) != 8192: raise RuntimeError('Unexpected probe SRAM size for '+title)
        previous=device.log().count('Snes9x auto state restored')
        device.key(96)
        wait_for(lambda:probe_visible(device.picture()),title+' resume')
        wait_for(lambda:device.log().count('Snes9x auto state restored')>previous,title+' verified state restoration')
        # Finish on the shelf, leaving the saved probe available for inspection.
        for key in [110,19,96]: device.key(key)
        wait_for(lambda:not probe_visible(device.picture()),title+' final eject')
        report['cases'].append({'title':title,'launch':True,'sram_bytes':len(battery),'state_written':True,'state_restored':True})
        print('PASS:',title,flush=True)
        (output/'report.json').write_text(json.dumps(report,indent=2)+'\n')
    (output/'logcat.txt').write_text(device.log())
    (output/'shelf.png').write_bytes(device.picture())
    print('All SNES Android smoke checks passed. Results:',output)


if __name__ == '__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--serial',default='emulator-5556')
    parser.add_argument('--rom-dir',default='/sdcard/Download/SlotSnesSmoke')
    parser.add_argument('--output',default='.build/snes-smoke')
    parser.add_argument('--prepare-only',action='store_true')
    args=parser.parse_args()
    try: run(args)
    except Exception as error:
        print('SNES smoke check failed:',error,file=sys.stderr); sys.exit(1)
