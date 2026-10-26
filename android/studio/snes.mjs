import { Drafts, coverRect } from './snes-state.mjs';
const $ = id => document.getElementById(id);
const data = JSON.parse(AndroidStudio.session());
const carts = data.carts.filter(c => c.platform === 'SNES');
const drafts = new Drafts(carts);
let selected = drafts.get(data.selectedKey) || drafts.get(carts[0]?.id);
let revision = 0, baseImage = null, busy = false;
const preview = $('preview'), ctx = preview.getContext('2d');
if (location.hash === '#embedded') document.body.classList.add('embedded');
if (data.selectedKey) { document.querySelector('.library').hidden = true; document.body.classList.add('single'); }
window.slotStudioHasChanges = () => drafts.pending().length > 0;

function actions() {
  const pending = drafts.pending();
  $('write').disabled = busy || pending.length === 0;
  $('discard').disabled = busy || !drafts.dirty(selected?.id);
  $('pending').textContent = pending.length ? `${pending.length} cartridge${pending.length === 1 ? '' : 's'} with unsaved changes` : '';
  // Hidden iframe must not replace the active platform's toolbar state.
  if (!window.frameElement?.hidden) window.AndroidToolbar?.actions(pending.length > 0, false);
  for (const button of $('colors').children) button.setAttribute('aria-pressed', String(selected?.shell === `auto ${button.dataset.color} solid`));
}
window.slotStudioActivate = actions;
function list() {
  const query = $('search').value.toLocaleLowerCase();
  const visible = carts.filter(c => c.stem.toLocaleLowerCase().includes(query));
  $('games').replaceChildren(...visible.map(c => Object.assign(document.createElement('option'), {value:c.id, textContent:c.stem})));
  $('games').value = selected?.id || '';
}
function draw() {
  ctx.clearRect(0, 0, 720, 480);
  if (baseImage) ctx.drawImage(baseImage, 0, 0, 720, 480);
  if (selected?.label) {
    ctx.save(); ctx.beginPath(); ctx.rect(132,108,456,276); ctx.clip();
    const [x,y,w,h] = coverRect(selected.label.width, selected.label.height);
    ctx.drawImage(selected.label,132+x,108+y,w,h); ctx.restore();
  }
}
async function decode(source) { const img = new Image(); img.src = source; await img.decode(); return img; }
async function refresh() {
  const token = ++revision, draft = selected;
  $('title').textContent = draft?.stem || 'No SNES cartridges';
  if (!draft) { actions(); return; }
  $('color').value = '#' + (draft.shell.split(' ')[1] || 'b6b5bb');
  try {
    const image = await decode('data:image/png;base64,' + AndroidStudio.snesPreview(draft.id, draft.shell));
    if (token !== revision) return;
    baseImage = image;
    if (!draft.loaded) {
      if (draft.hasLabel) {
        const encoded = AndroidStudio.readLabel(draft.id);
        if (encoded.startsWith('error:')) throw Error(encoded);
        if (encoded) { const saved = await decode('data:image/png;base64,' + encoded); if (!draft.labelDirty) draft.label = saved; }
      }
      draft.loaded = true;
    }
    if (token === revision) draw();
  } catch (error) { if (token === revision) $('status').textContent = error.message; }
  actions();
}
$('search').oninput = list;
$('games').onchange = () => {
  if (!drafts.get($('games').value)) return;
  selected = drafts.get($('games').value); $('status').textContent = ''; refresh();
};
function shell(value) { if (selected) { selected.shell = value; refresh(); } }
for (const color of ['b6b5bb','504360','24242c','e4ded2','b7444d','365d9d','447e64','d6aa52']) {
  const button = document.createElement('button'); button.type = 'button';
  button.dataset.color = color; button.style.background = '#' + color;
  button.setAttribute('aria-label', `Shell color #${color}`);
  button.onclick = () => shell(`auto ${color} solid`); $('colors').append(button);
}
$('color').onchange = () => shell(`auto ${$('color').value.slice(1)} solid`);
$('automatic').onclick = () => shell('auto');
$('pick').onclick = () => $('file').click();
$('file').onchange = async e => {
  const file = e.target.files[0], draft = selected;
  if (!file || !draft) return;
  try {
    if (file.size > 9*1024*1024) throw Error('Choose a PNG smaller than 9 MiB.');
    const signature = new Uint8Array(await file.slice(0,8).arrayBuffer());
    if (signature.join() !== '137,80,78,71,13,10,26,10') throw Error('Choose a PNG image.');
    const url = URL.createObjectURL(file); let image;
    try { image = await decode(url); } finally { URL.revokeObjectURL(url); }
    if (image.width > 4096 || image.height > 4096) throw Error('Choose an image up to 4096 × 4096 pixels.');
    draft.label = image; draft.labelDirty = true; draft.loaded = true;
    if (selected === draft) { draw(); $('status').textContent = 'Label ready to save.'; }
    actions();
  } catch (error) { $('status').textContent = error.message; }
  e.target.value = '';
};
$('discard').onclick = () => { if (selected) { drafts.discard(selected.id); $('status').textContent = 'Changes discarded.'; refresh(); } };
$('write').onclick = async () => {
  if (busy) return;
  busy = true; actions(); let saved = 0;
  try {
    for (const draft of drafts.pending()) {
      if (draft.labelDirty) {
        const output = document.createElement('canvas'); output.width = 456; output.height = 276;
        output.getContext('2d').drawImage(draft.label,...coverRect(draft.label.width,draft.label.height));
        const result = AndroidStudio.saveSnesLabel(draft.id,output.toDataURL('image/png').split(',')[1]);
        if (result !== 'written') throw Error(result);
        drafts.savedLabel(draft.id);
      }
      if (draft.shell !== draft.savedShell) {
        const result = AndroidStudio.saveSnesShell(draft.id,draft.shell);
        if (result !== 'written') throw Error(result);
        drafts.savedShell(draft.id);
      }
      saved++;
    }
    $('status').textContent = `Saved ${saved} cartridge${saved === 1 ? '' : 's'}.`;
  } catch (error) { $('status').textContent = `${error.message} Unsaved changes are kept.`; }
  finally { busy = false; actions(); window.AndroidToolbar?.progress(0,0); }
};
list(); refresh().then(() => { document.body.dataset.ready = 'true'; $(data.selectedKey ? 'pick' : 'games').focus(); }); actions();
