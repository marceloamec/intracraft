// Inline toolbar icons (our own simple drawings, no icon library dependency).
const PATHS = {
  newLayout: 'M6 3h8l4 4v14H6z M14 3v4h4 M12 11v6 M9 14h6',
  undo: 'M9 7L4 12l5 5 M4 12h10a6 6 0 0 1 0 12h-2',
  redo: 'M15 7l5 5-5 5 M20 12H10a6 6 0 0 0 0 12h2',
  rotate: 'M20 12a8 8 0 1 1-2.3-5.7 M20 4v5h-5',
  remove: 'M5 7h14 M10 7V4h4v3 M7 7l1 13h8l1-13 M10 11v6 M14 11v6',
  view3d: 'M12 3l8 4.5v9L12 21l-8-4.5v-9z M12 12l8-4.5 M12 12v9 M12 12L4 7.5',
  viewTop: 'M4 4h16v16H4z M4 12h16 M12 4v16',
  components: 'M4 4h7v7H4z M13 4h7v7h-7z M4 13h7v7H4z M13 13h7v7h-7z',
  properties: 'M4 6h16 M4 12h16 M4 18h16 M8 4v4 M16 10v4 M10 16v4',
  assets: 'M3 8l9-5 9 5-9 5z M3 8v8l9 5 9-5V8 M12 13v8',
  open: 'M3 7h6l2 2h10v10H3z',
  frameAll: 'M4 9V4h5 M15 4h5v5 M20 15v5h-5 M9 20H4v-5 M9 9h6v6H9z',
  save: 'M5 3h11l3 3v15H5z M8 3v6h8V3 M8 21v-7h8v7',
  controller: 'M4 6h6v4H4z M14 14h6v4h-6z M10 8h4v8h0 M14 16h-4',
  play: 'M7 4l13 8-13 8z',
  pause: 'M7 4h4v16H7z M14 4h4v16h-4z',
  step: 'M5 4l10 8-10 8z M18 4v16',
  stop: 'M6 6h12v12H6z',
} as const;

export type IconName = keyof typeof PATHS;

export function Icon({ name }: { name: IconName }) {
  return (
    <svg
      viewBox="0 0 24 24"
      width="18"
      height="18"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.8"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
    >
      <path d={PATHS[name]} />
    </svg>
  );
}
