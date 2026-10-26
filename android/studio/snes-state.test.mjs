import test from 'node:test';
import assert from 'node:assert/strict';
import { Drafts, coverRect } from './snes-state.mjs';
test('duplicate titles keep independent drafts across selection and partial saves', () => {
  const drafts = new Drafts([{id:'us',stem:'Game',shell:'auto'}, {id:'jp',stem:'Game',shell:'auto'}]);
  drafts.get('us').shell = 'auto ff0000 solid';
  drafts.get('us').labelDirty = true;
  drafts.get('jp').shell = 'auto 0000ff solid';
  drafts.savedLabel('us');
  assert.equal(drafts.dirty('us'),true); // Failed shell write must remain pending.
  assert.equal(drafts.get('us').hasLabel,true);
  drafts.savedShell('us');
  assert.deepEqual(drafts.pending().map(d=>d.id),['jp']);
  drafts.discard('jp');
  assert.equal(drafts.get('jp').shell,'auto');
  assert.equal(drafts.pending().length,0);
});
test('cover crop preserves aspect and fills the native 228x138 label at 2x', () => {
  assert.deepEqual(coverRect(456,276),[0,0,456,276]);
  for (const [w,h] of [[100,400],[1000,100],[720,480]]) {
    const [x,y,cw,ch]=coverRect(w,h);
    assert.ok(cw>=456-1e-9 && ch>=276-1e-9 && x<=1e-9 && y<=1e-9);
    assert.ok(Math.abs(cw/ch-w/h)<1e-12);
    assert.equal(x*2+cw,456); assert.equal(y*2+ch,276);
  }
  assert.throws(()=>coverRect(0,10));
});
