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
