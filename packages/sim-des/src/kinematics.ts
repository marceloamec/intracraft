// Analytic kinematics used by the 2D (DES) mode. All values in SI units (m, s, rad).
// The same formulas apply to linear moves (m, m/s, m/s²) and rotations (rad, rad/s, rad/s²).

export interface MoveProfile {
  /** Distance (or angle) to cover, ≥ 0. */
  distance: number;
  /** Cruise speed, > 0. */
  maxSpeed: number;
  /** Acceleration ramp from rest, > 0. */
  accel: number;
  /** Deceleration ramp to rest at the end. Omit to arrive at cruise speed (no braking). */
  decel?: number;
}

/** A constant-acceleration piece of motion: s(t) = s0 + v0·(t − t0) + ½·a·(t − t0)². */
export interface KinematicSegment {
  t0: number;
  s0: number;
  v0: number;
  a: number;
}

/**
 * Duration of a move that starts at rest, accelerates to maxSpeed, cruises and optionally
 * decelerates to rest. Falls back to a triangular profile when the distance is too short
 * to reach cruise speed.
 */
export function profileTime({ distance, maxSpeed, accel, decel }: MoveProfile): number {
  assertPositive({ maxSpeed, accel });
  if (distance < 0) throw new Error(`distance must be ≥ 0, got ${distance}`);
  if (distance === 0) return 0;

  const accelDistance = (maxSpeed * maxSpeed) / (2 * accel);
  const decelDistance = decel === undefined ? 0 : (maxSpeed * maxSpeed) / (2 * decel);

  if (distance >= accelDistance + decelDistance) {
    const accelTime = maxSpeed / accel;
    const decelTime = decel === undefined ? 0 : maxSpeed / decel;
    const cruiseTime = (distance - accelDistance - decelDistance) / maxSpeed;
    return accelTime + cruiseTime + decelTime;
  }

  if (decel === undefined) return Math.sqrt((2 * distance) / accel);
  const peakSpeed = Math.sqrt((2 * distance * accel * decel) / (accel + decel));
  return peakSpeed / accel + peakSpeed / decel;
}

/** Transfer-table cycle: lift + chain move (accelerate, cruise, brake) + lower. */
export function transferCycleTime(args: {
  liftTime: number;
  lowerTime: number;
  distance: number;
  speed: number;
  accel: number;
  settleTime?: number;
}): number {
  const move = profileTime({
    distance: args.distance,
    maxSpeed: args.speed,
    accel: args.accel,
    decel: args.accel,
  });
  return args.liftTime + move + args.lowerTime + (args.settleTime ?? 0);
}

/** Maximum steady throughput (pallets/s) of a line limited by spacing: v / (L + gap). */
export function maxThroughput(speed: number, palletLength: number, gap: number): number {
  return speed / (palletLength + gap);
}

/** Number of pallets that fit on a conveyor: ⌊length / (L + gap)⌋. */
export function palletCapacity(length: number, palletLength: number, gap: number): number {
  return Math.floor(length / (palletLength + gap));
}

/** Kinetic energy at impact: E = ½·m·v² (J). */
export function impactEnergy(mass: number, speed: number): number {
  return 0.5 * mass * speed * speed;
}

export function positionAt(seg: KinematicSegment, t: number): number {
  const dt = t - seg.t0;
  return seg.s0 + seg.v0 * dt + 0.5 * seg.a * dt * dt;
}

export function velocityAt(seg: KinematicSegment, t: number): number {
  return seg.v0 + seg.a * (t - seg.t0);
}

/**
 * Earliest time ≥ t0 at which the segment reaches position s, or undefined if it never does.
 * This is how the DES engine predicts events such as "front reaches sensor".
 */
export function timeToReach(seg: KinematicSegment, s: number): number | undefined {
  const ds = s - seg.s0;
  if (ds === 0) return seg.t0;
  if (seg.a === 0) {
    if (seg.v0 === 0) return undefined;
    const dt = ds / seg.v0;
    return dt >= 0 ? seg.t0 + dt : undefined;
  }
  // ½·a·dt² + v0·dt − ds = 0
  const disc = seg.v0 * seg.v0 + 2 * seg.a * ds;
  if (disc < 0) return undefined;
  const root = Math.sqrt(disc);
  const candidates = [(-seg.v0 + root) / seg.a, (-seg.v0 - root) / seg.a].filter((dt) => dt >= 0);
  if (candidates.length === 0) return undefined;
  return seg.t0 + Math.min(...candidates);
}

function assertPositive(values: Record<string, number>): void {
  for (const [name, value] of Object.entries(values)) {
    if (!(value > 0)) throw new Error(`${name} must be > 0, got ${value}`);
  }
}
