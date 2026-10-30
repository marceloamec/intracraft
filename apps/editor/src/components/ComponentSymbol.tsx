// Top-view symbols of the library components (our own simple drawings).
const STROKE = { stroke: 'currentColor', strokeWidth: 1.5, fill: 'none' } as const;

export function ComponentSymbol({ definitionId }: { definitionId: string }) {
  return (
    <svg viewBox="0 0 48 32" width="48" height="32" aria-hidden="true">
      {SYMBOLS[definitionId] ?? <rect x="4" y="6" width="40" height="20" {...STROKE} />}
    </svg>
  );
}

const rollers = (x0: number, x1: number, step: number) =>
  Array.from({ length: Math.floor((x1 - x0) / step) + 1 }, (_, i) => x0 + i * step).map((x) => (
    <line key={x} x1={x} y1="8" x2={x} y2="24" {...STROKE} strokeWidth={1} />
  ));

const SYMBOLS: Record<string, React.JSX.Element> = {
  'conveyor.roller': (
    <>
      <rect x="2" y="6" width="44" height="20" {...STROKE} />
      {rollers(6, 42, 4)}
    </>
  ),
  'conveyor.chain': (
    <>
      <rect x="2" y="4" width="44" height="24" {...STROKE} />
      {[10, 16, 22].map((y) => (
        <line key={y} x1="4" y1={y} x2="44" y2={y} {...STROKE} strokeWidth={2.5} />
      ))}
    </>
  ),
  'conveyor.chain_turntable': (
    <>
      <circle cx="24" cy="16" r="14" {...STROKE} strokeDasharray="2 2" />
      <rect x="13" y="7" width="22" height="18" {...STROKE} />
      {[11, 16, 21].map((y) => (
        <line key={y} x1="14" y1={y} x2="34" y2={y} {...STROKE} strokeWidth={2} />
      ))}
    </>
  ),
  'conveyor.roller_turntable': (
    <>
      <circle cx="24" cy="16" r="14" {...STROKE} strokeDasharray="2 2" />
      <rect x="13" y="7" width="22" height="18" {...STROKE} />
      {[16, 20, 24, 28, 32].map((x) => (
        <line key={x} x1={x} y1="8" x2={x} y2="24" {...STROKE} strokeWidth={1} />
      ))}
    </>
  ),
  'flow.source': (
    <>
      <rect x="4" y="6" width="16" height="20" {...STROKE} />
      <path d="M22 16 H42 M36 10 L42 16 L36 22" {...STROKE} />
    </>
  ),
  'flow.sink': (
    <>
      <path d="M4 16 H24 M18 10 L24 16 L18 22" {...STROKE} />
      <rect x="28" y="6" width="16" height="20" {...STROKE} />
      <path d="M31 9 L41 23 M41 9 L31 23" {...STROKE} strokeWidth={1} />
    </>
  ),
  'flow.process_station': (
    <>
      <rect x="2" y="6" width="44" height="20" {...STROKE} />
      {rollers(6, 42, 6)}
      <rect x="18" y="2" width="12" height="28" {...STROKE} strokeWidth={2.5} />
    </>
  ),
  'sensor.photoeye': (
    <>
      <rect x="4" y="12" width="8" height="8" {...STROKE} />
      <path d="M12 16 H36" {...STROKE} strokeDasharray="3 2" />
      <rect x="36" y="11" width="4" height="10" {...STROKE} />
    </>
  ),
  'conveyor.lift_table_manual': (
    <>
      <rect x="14" y="6" width="32" height="20" {...STROKE} />
      {rollers(18, 42, 4)}
      {/* Pallet jack forks on the operator side. */}
      <path d="M2 11 H12 M2 21 H12 M2 9 V23" {...STROKE} strokeWidth={2} />
    </>
  ),
  'conveyor.scissor_lift': (
    <>
      <rect x="2" y="6" width="44" height="20" {...STROKE} />
      {rollers(6, 42, 6)}
      <path d="M10 8 L38 24 M10 24 L38 8" {...STROKE} strokeWidth={2} />
    </>
  ),
  'conveyor.elevator': (
    <>
      <rect x="8" y="7" width="32" height="18" {...STROKE} />
      {rollers(12, 36, 6)}
      <rect x="20" y="1" width="8" height="5" {...STROKE} strokeWidth={2} />
      <rect x="20" y="26" width="8" height="5" {...STROKE} strokeWidth={2} />
      <path d="M44 22 V10 M41 13 L44 10 L47 13" {...STROKE} />
    </>
  ),
  'conveyor.transfer_rc': (
    <>
      <rect x="8" y="4" width="32" height="24" {...STROKE} />
      {rollers(12, 36, 4)}
      {[16, 24, 32].map((x) => (
        <line key={x} x1={x} y1="2" x2={x} y2="30" {...STROKE} strokeWidth={2.5} />
      ))}
    </>
  ),
};
