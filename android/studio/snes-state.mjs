// Per-cartridge drafts survive selection changes and mixed-system tab switches.
export class Drafts {
  constructor(carts) {
    this.items = new Map(carts.map(cart => [cart.id, {
      ...cart, savedShell: cart.shell || 'auto', shell: cart.shell || 'auto',
      label: null, labelDirty: false, loaded: false,
    }]));
  }
  get(id) { return this.items.get(id); }
  dirty(id) { const d = this.get(id); return !!d && (d.labelDirty || d.shell !== d.savedShell); }
  pending() { return [...this.items.values()].filter(d => this.dirty(d.id)); }
  savedLabel(id) { const d = this.get(id); d.labelDirty = false; d.hasLabel = true; }
  savedShell(id) { const d = this.get(id); d.savedShell = d.shell; }
  discard(id) {
    const d = this.get(id); d.shell = d.savedShell;
    d.label = null; d.labelDirty = false; d.loaded = false;
  }
}

export function coverRect(width, height, targetWidth = 456, targetHeight = 276) {
  if (!(width > 0 && height > 0)) throw Error('Invalid image dimensions.');
  const scale = Math.max(targetWidth / width, targetHeight / height);
  return [(targetWidth - width * scale) / 2, (targetHeight - height * scale) / 2,
    width * scale, height * scale];
}
