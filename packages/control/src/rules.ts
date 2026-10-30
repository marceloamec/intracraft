// If/Then rules over boolean signals. The evaluator is pure: it receives the previous and the
// current signal values and returns the writes to apply, so 2D (event-driven) and 3D (cyclic,
// every 10 ms) evaluation share exactly the same logic. Delays are scheduled by the caller.

export type SignalValues = Readonly<Record<string, boolean>>;

export type Condition =
  | { signal: string; is: boolean }
  | { signal: string; edge: 'rising' | 'falling' };

export interface SignalWrite {
  signal: string;
  value: boolean;
  /** Delay before the write takes effect (ms). */
  delayMs: number;
  ruleId: string;
}

export interface Rule {
  id: string;
  /** All conditions must hold (logical AND). */
  when: Condition[];
  then: { signal: string; value: boolean; delayMs?: number | undefined }[];
}

export function evaluateRules(
  rules: readonly Rule[],
  previous: SignalValues,
  current: SignalValues,
): SignalWrite[] {
  const writes: SignalWrite[] = [];
  for (const rule of rules) {
    if (!rule.when.every((c) => conditionHolds(c, previous, current))) continue;
    for (const action of rule.then) {
      writes.push({
        signal: action.signal,
        value: action.value,
        delayMs: action.delayMs ?? 0,
        ruleId: rule.id,
      });
    }
  }
  return writes;
}

function conditionHolds(c: Condition, previous: SignalValues, current: SignalValues): boolean {
  const now = current[c.signal] ?? false;
  if ('is' in c) return now === c.is;
  const before = previous[c.signal] ?? false;
  return c.edge === 'rising' ? !before && now : before && !now;
}
