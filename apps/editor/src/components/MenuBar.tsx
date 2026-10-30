// Application menu: File, Edit, View, Run. Items are the shared editor actions (actions.ts).
import { useEffect, useRef, useState } from 'react';
import { type Action, useActions } from '../actions';
import type { StringKey } from '../i18n/strings';
import { useT } from '../i18n/useT';

export function MenuBar() {
  const t = useT();
  const a = useActions();
  const [open, setOpen] = useState<string | null>(null);
  const bar = useRef<HTMLElement>(null);

  useEffect(() => {
    if (open === null) return;
    const close = (e: PointerEvent) => {
      if (bar.current !== null && !bar.current.contains(e.target as Node)) setOpen(null);
    };
    window.addEventListener('pointerdown', close);
    return () => window.removeEventListener('pointerdown', close);
  }, [open]);

  const menus: { id: string; label: StringKey; items: (Action | 'separator')[] }[] = [
    { id: 'file', label: 'menuFile', items: [a.newLayout, a.open, a.save, 'separator', a.exportResults] },
    { id: 'edit', label: 'menuEdit', items: [a.undo, a.redo, 'separator', a.rotate, a.remove] },
    { id: 'view', label: 'menuView', items: [a.toggleView, a.frameAll, 'separator', a.components, a.properties, a.controller, a.assets] },
    { id: 'run', label: 'menuRun', items: [a.playPause, a.step, a.stop] },
  ];

  return (
    <nav className="menubar" ref={bar}>
      <strong className="app-title">{t('appTitle')}</strong>
      {menus.map((menu) => (
        <div key={menu.id} className="menu">
          <button
            type="button"
            className="menu-button"
            aria-expanded={open === menu.id}
            onClick={() => setOpen(open === menu.id ? null : menu.id)}
            onPointerEnter={() => open !== null && setOpen(menu.id)}
          >
            {t(menu.label)}
          </button>
          {open === menu.id && (
            <ul className="menu-list" role="menu">
              {menu.items.map((item, i) =>
                item === 'separator' ? (
                  <li key={i} className="menu-separator" role="separator" />
                ) : (
                  <li key={item.id} role="none">
                    <button
                      type="button"
                      role={item.checked === undefined ? 'menuitem' : 'menuitemcheckbox'}
                      aria-checked={item.checked}
                      disabled={!item.enabled}
                      onClick={() => {
                        setOpen(null);
                        item.run();
                      }}
                    >
                      <span className="menu-check">{item.checked === true ? '✓' : ''}</span>
                      <span>{t(item.label)}</span>
                      <kbd>{item.shortcut ?? ''}</kbd>
                    </button>
                  </li>
                ),
              )}
            </ul>
          )}
        </div>
      ))}
    </nav>
  );
}
