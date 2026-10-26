// Keep both editors alive while switching systems: pending edits stay in memory.
export function installSnesTabs({showPlatform, selectedPlatform, hasChanges, refreshToolbar}) {
  const carts = JSON.parse(AndroidStudio.session()).carts;
  const snes = carts.filter(c => c.platform === 'SNES');
  let frame = null;
  window.slotStudioHasChanges = () => hasChanges() || !!frame?.contentWindow?.slotStudioHasChanges?.();
  window.slotStudioDocument = () => frame && !frame.hidden ? frame.contentDocument : document;
  const api = {
    render() {
      const tabs = document.getElementById('tabs');
      if (!snes.length || !tabs) return;
      tabs.hidden = false;
      const button = document.createElement('button');
      button.type = 'button'; button.className = 'shelf-tab';
      button.append('Super Nintendo', Object.assign(document.createElement('span'), {className:'n',textContent:snes.length}));
      button.setAttribute('aria-pressed',String(selectedPlatform() === 'SNES'));
      button.onclick = () => showPlatform('SNES'); tabs.append(button);
    },
    show(platform) {
      const active = platform === 'SNES';
      const grid = document.getElementById('grid');
      grid.hidden = active;
      if (active && !frame) {
        frame = document.createElement('iframe'); frame.id = 'snes-frame';
        frame.title = 'Super Nintendo cartridges'; frame.src = 'snes.html#embedded';
        frame.style.cssText = 'display:block;width:100%;height:520px;border:0;background:#1b1b21';
        // The hidden attribute must override the inline display value.
        frame.addEventListener('load',()=>frame.contentWindow?.slotStudioActivate?.());
        grid.after(frame);
      }
      if (frame) { frame.hidden = !active; frame.style.display = active ? 'block' : 'none'; }
      if (active) {
        document.getElementById('write-bar').hidden = true;
        document.getElementById('fill-open').hidden = true;
        window.AndroidToolbar?.actions(false,false);
        frame.contentWindow?.slotStudioActivate?.();
      } else refreshToolbar();
    },
  };
  window.slotSnesTabs = api; api.render();
}
