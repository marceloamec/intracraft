// 2D behaviour of the photoelectric sensor (S06, T06). The beam is the segment sensor → reflector
// in the host conveyor's frame. Each solid part of a pallet (its occupancy profile at the beam
// height, or the load above it) blocks the beam while the pallet front lies in one interval;
// the intervals are exact, so the flow simulation can predict every edge.
import { type SimTime, secondsToSimTime } from '@intracraft/core-model';
import { PALLET_TYPES } from '@intracraft/library';
import { EPS } from './line-conveyor';
import type { SensorNode } from './network';
import type { Pallet } from './pallet';

export type Interval = [from: number, to: number];

export class Photoeye {
  /** The beam is interrupted right now. */
  raw = false;
  /** Sensor output (detection), after response time and filter. */
  detecting = false;
  pendingTicket: number | undefined;
  readonly stats = { pulses: 0, occupied: 0, total: 0 };
  private readonly cache = new Map<string, Interval[]>();

  /** Name of the output signal: `outputSignal`, or `<id>.detected`. */
  readonly signal: string;

  constructor(readonly node: SensorNode) {
    this.signal = node.outputSignal === '' ? `${node.id}.detected` : node.outputSignal;
  }

  /** Output signal value: detection, inverted for "active on dark". */
  get signalValue(): boolean {
    return this.detecting !== this.node.activeOnDark;
  }

  /** Front positions (m, host frame) at which `pallet` interrupts the beam, sorted and merged. */
  intervals(pallet: Pallet): Interval[] {
    const key = `${pallet.typeId}|${pallet.along}|${pallet.length}|${pallet.width}|${pallet.height}|${pallet.loadHeight}`;
    const cached = this.cache.get(key);
    if (cached !== undefined) return cached;
    const crossing = beamSpan(this.node, pallet.width / 2);
    const solid = crossing === undefined ? [] : solidParts(pallet, this.node.beamHeight);
    const result = crossing === undefined ? [] : merge(solid.map(([a, b]) => [crossing[0] + a, crossing[1] + b]));
    this.cache.set(key, result);
    return result;
  }

  blockedBy(pallet: Pallet, front: number): boolean {
    return this.intervals(pallet).some(([from, to]) => front >= from - EPS && front < to - EPS);
  }

  /**
   * Delay before the output may follow the beam (µs): the response time, plus the filter when the
   * beam becomes free. The filter is an off-delay, so short voids in a pallet do not drop the output.
   */
  delayFor(blocked: boolean): SimTime {
    return secondsToSimTime(this.node.responseTime + (blocked ? 0 : this.node.filter));
  }

  /**
   * The output takes the beam state. Called at once without delay, else `delayFor` after the last
   * change: every change cancels the pending call, so a free beam has lasted at least the filter time.
   */
  commit(state: boolean): void {
    if (state === this.detecting) return;
    if (state) this.stats.pulses += 1;
    this.detecting = state;
  }

  accumulate(seconds: number): void {
    this.stats.total += seconds;
    if (this.detecting) this.stats.occupied += seconds;
  }

  statistics(): Record<string, number> {
    return {
      pulses: this.stats.pulses,
      occupiedTime: this.stats.occupied,
      occupancy: this.stats.total > 0 ? this.stats.occupied / this.stats.total : 0,
    };
  }
}

/**
 * Range of positions along travel where the beam lies over the pallet's width (|lateral| ≤ half
 * width), or undefined when the beam passes beside the pallet.
 */
export function beamSpan(node: SensorNode, halfWidth: number): Interval | undefined {
  const { sensorS: s1, sensorLateral: z1, reflectorS: s2, reflectorLateral: z2 } = node;
  let from = 0;
  let to = 1;
  if (z1 === z2) {
    if (Math.abs(z1) > halfWidth) return undefined;
  } else {
    const a = (-halfWidth - z1) / (z2 - z1);
    const b = (halfWidth - z1) / (z2 - z1);
    from = Math.max(0, Math.min(a, b));
    to = Math.min(1, Math.max(a, b));
    if (from > to) return undefined;
  }
  const sFrom = s1 + from * (s2 - s1);
  const sTo = s1 + to * (s2 - s1);
  return [Math.min(sFrom, sTo), Math.max(sFrom, sTo)];
}

/** Solid parts (m from the pallet front) that a horizontal beam at `height` meets. */
export function solidParts(pallet: Pallet, height: number): Interval[] {
  if (height < 0) return [];
  if (height >= pallet.height) return height < pallet.height + pallet.loadHeight ? [[0, pallet.length]] : [];
  const type = PALLET_TYPES.find((p) => p.id === pallet.typeId);
  const bands = type?.occupancy[pallet.along === 'long' ? 'alongLength' : 'alongWidth'] ?? [];
  const band = bands.find((b) => height >= b.fromHeight && height < b.toHeight);
  // Without a profile the pallet is treated as solid.
  return band === undefined ? [[0, pallet.length]] : band.solid.map(([a, b]) => [a, b]);
}

function merge(intervals: Interval[]): Interval[] {
  const sorted = [...intervals].sort((a, b) => a[0] - b[0]);
  const result: Interval[] = [];
  for (const [from, to] of sorted) {
    const last = result.at(-1);
    if (last !== undefined && from <= last[1]) last[1] = Math.max(last[1], to);
    else result.push([from, to]);
  }
  return result;
}
