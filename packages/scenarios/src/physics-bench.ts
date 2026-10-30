// Physics test bench: a straight conveyor (rollers, or chain strands by plan B or plan A) with an
// optional fixed stop, side guides with a funnel, or a turntable platform, and one MTBH built from
// real models, run with the Jolt backend at the fixed step. Shared by the reference scenarios P1–P11.
// Bench frame: X along travel from the conveyor start, Y up, Z across; the floor is at y = 0. Extra
// lanes (performance runs) are copies of lane 0 shifted along Z.
import { builtInCatalog } from '@intracraft/asset-catalog';
import type { Vec3 } from '@intracraft/core-model';
import {
  PALLET_TYPES,
  type ParamValues,
  type ParameterDefinition,
  contactPair,
  num,
  parameterSchema,
} from '@intracraft/library';
import {
  type BodyHandle,
  DEFAULT_PHYSICS_DT,
  type ModelPlacement,
  type Mtbh,
  type PhysicsBackend,
  type Pose,
  type Shape,
  type Velocity,
  buildMtbh,
  createJoltBackend,
  palletConstruction,
  rollerAngularVelocity,
} from '@intracraft/sim-physics';
import { z } from 'zod';
import benchJson from '../scenarios/physics-bench.json';

export const BENCH_PARAMETERS: ParameterDefinition[] = z.array(parameterSchema).parse(benchJson);

/** Height of the roller tops above the floor (m). */
export const BENCH_SURFACE_HEIGHT = 0.5;
/** Roller length across the conveyor (m). */
export const BENCH_USABLE_WIDTH = 1;
/** The pallet starts with its rear this far from the conveyor start (m). */
const START_GAP = 0.05;
const STOP_THICKNESS = 0.02;
/** The stop reaches this far below the conveying surface, between the rollers (m). */
const STOP_DEPTH = 0.03;
/** Chain strand cross-section (m), as the chain support of S02. */
const STRAND_WIDTH = 0.04;
const STRAND_HEIGHT = 0.03;
/** Chain plan A: small driven rollers under each strand (m). */
const PLAN_A_DIAMETER = 0.03;
const PLAN_A_PITCH = 0.04;
const GUIDE_THICKNESS = 0.01;
/** The guides start this far above the conveying surface (m). */
const GUIDE_GAP = 0.005;
/** Distance between the axes of neighbouring lanes (m). */
const LANE_SPACING = 1.5;
const FLOOR_HALF = { x: 10, y: 0.05, z: 40 };
const ZERO: Vec3 = { x: 0, y: 0, z: 0 };

/** Pallet and load combinations offered by the bench, from the asset catalogue (ADR-0015). */
export const MTBH_TYPES = {
  'eur-wood-cartons': {
    pallet: 'asset.pallets.pallet_block_a07',
    load: 'asset.pallets.pallet_composition_a59',
    palletType: 'eur',
    material: 'wood',
  },
  'eur-wood': { pallet: 'asset.pallets.pallet_block_a07', palletType: 'eur', material: 'wood' },
  plastic: {
    pallet: 'asset.pallets.pallet_rough_black_a02',
    palletType: 'eur',
    material: 'plastic',
  },
} as const satisfies Record<
  string,
  { pallet: string; load?: string; palletType: string; material: string }
>;

export type MtbhType = keyof typeof MTBH_TYPES;

export type BenchDeck = 'rollers' | 'chains' | 'chainsPlanA';

/** A box placed in the bench frame and turned about Y by `yaw` (rad). */
export interface BenchBox {
  center: Vec3;
  halfExtents: Vec3;
  yaw: number;
  material: string;
}

/** Static description of the bench (lane 0), for drawing it. */
export interface BenchLayout {
  conveyorLength: number;
  surfaceHeight: number;
  usableWidth: number;
  deck: BenchDeck;
  /** Driven rollers; their axes run along Z. Chain plan A: short rollers under each strand. */
  rollers: { x: number; y: number; z: number; diameter: number; length: number }[];
  /** Chain strands along X, their top at the surface height. */
  strands: { z: number; width: number }[];
  /** Guide rails (yaw 0) and funnel plates. */
  guides: BenchBox[];
  /** Turntable: the rollers are mounted on a platform turning about this point. */
  platform?: { center: Vec3 };
  stop?: { center: Vec3; halfExtents: Vec3; material: string };
  /** Z of each lane's axis; lane 0 is at 0. */
  laneOffsets: number[];
  mtbh: {
    /** Asset drawn for the MTBH and where it sits in the MTBH body frame. */
    visualAsset: string;
    placement: ModelPlacement;
    length: number;
    width: number;
    height: number;
    mass: number;
    /** Body frame: X along the pallet length, origin at the footprint centre on the underside. */
    centerOfMass: Vec3;
    /** Start position of the body origin on the floor plane and its yaw (rad). */
    start: { x: number; z: number; yaw: number };
  };
}

export interface BenchImpact {
  time: number;
  speed: number;
  energy: number;
}

/** State of the bench after a step (pallet of lane 0). */
export interface BenchFrame {
  time: number;
  rollerSpeed: number;
  /** Accumulated roller rotation (rad), for drawing. */
  rollerAngle: number;
  pallet: { pose: Pose; velocity: Velocity };
  /** X of the pallet's front edge (m). */
  front: number;
  /** Turn of the pallet about the vertical from its travel-aligned orientation (rad). */
  yaw: number;
  /** Roller surface speed minus pallet speed along travel (m/s). */
  slip: number;
  /** Angle between the pallet's up axis and the vertical (rad). */
  tilt: number;
  /** Turntable platform angle (rad), 0 without a turntable. */
  platformAngle: number;
  /** Pallet impacts against the stop so far. */
  impacts: BenchImpact[];
  /** Largest backward travel after the first stop impact (m). */
  rebound: number;
  /** The pallet has left the conveyor and dropped. */
  fallen: boolean;
}

interface BenchBodies {
  rollers: BodyHandle[];
  /** Driven rollers of chain plan A. */
  smallRollers: BodyHandle[];
  strands: BodyHandle[];
  platforms: BodyHandle[];
  stops: BodyHandle[];
  pallets: BodyHandle[];
}

export class PhysicsBench {
  readonly dt = DEFAULT_PHYSICS_DT;
  private time = 0;
  private rollerAngle = 0;
  private readonly impacts: BenchImpact[] = [];
  private furthestFront = -Infinity;
  private rebound = 0;

  private constructor(
    readonly layout: BenchLayout,
    private readonly params: ParamValues,
    private readonly world: PhysicsBackend,
    private readonly bodies: BenchBodies,
  ) {}

  /** Builds the bench from parameter values (missing keys take the defaults of BENCH_PARAMETERS). */
  static async create(values: ParamValues = {}): Promise<PhysicsBench> {
    const params = { ...benchDefaults(), ...values };
    const world = await createJoltBackend({ contactPair });
    world.addBody({
      motion: 'static',
      pose: pose({ x: num(params, 'conveyorLength', 3) / 2, y: -FLOOR_HALF.y, z: 0 }),
      shapes: [{ kind: 'box', halfExtents: FLOOR_HALF, offset: ZERO }],
      material: 'concrete',
    });

    const mtbh = buildBenchMtbh(params);
    const layout = benchLayout(params, mtbh);
    const bodies: BenchBodies = {
      rollers: [],
      smallRollers: [],
      strands: [],
      platforms: [],
      stops: [],
      pallets: [],
    };
    for (const [lane, laneZ] of layout.laneOffsets.entries()) {
      addLane(world, layout, params, laneZ, bodies);
      if (lane < num(params, 'loadedLanes', 1))
        bodies.pallets.push(addPallet(world, layout, mtbh, laneZ));
    }
    const first = bodies.pallets[0];
    if (params.startAtSpeed === true && first !== undefined)
      world.setVelocity(first, { x: num(params, 'speed', 0.25), y: 0, z: 0 }, ZERO);
    return new PhysicsBench(layout, params, world, bodies);
  }

  /** Bodies driven by the bench, for performance figures. */
  get bodyCount(): { rollers: number; pallets: number } {
    return {
      rollers: this.bodies.rollers.length + this.bodies.smallRollers.length,
      pallets: this.bodies.pallets.length,
    };
  }

  /** Advances one fixed step and returns the new state. */
  step(): BenchFrame {
    const speed = this.rollerSpeedAt(this.time);
    const omega = rollerAngularVelocity(speed, num(this.params, 'rollerDiameter', 0.089));
    for (const roller of this.bodies.rollers)
      this.world.setVelocity(roller, ZERO, { x: 0, y: 0, z: -omega });
    const smallOmega = rollerAngularVelocity(speed, PLAN_A_DIAMETER);
    for (const roller of this.bodies.smallRollers)
      this.world.setVelocity(roller, ZERO, { x: 0, y: 0, z: -smallOmega });
    for (const strand of this.bodies.strands)
      this.world.setSurfaceVelocity(strand, { x: speed, y: 0, z: 0 });
    const turn = this.platformSpeedAt(this.time + this.dt / 2);
    for (const platform of this.bodies.platforms)
      this.world.setVelocity(platform, ZERO, { x: 0, y: turn, z: 0 });
    this.world.step(this.dt);
    this.time += this.dt;
    if (this.layout.deck === 'rollers' && this.bodies.platforms.length === 0)
      this.rollerAngle += omega * this.dt;

    const pallet = this.bodies.pallets[0];
    const stop = this.bodies.stops[0];
    for (const impact of this.world.takeImpacts()) {
      if (
        stop !== undefined &&
        pallet !== undefined &&
        impact.bodies.includes(stop) &&
        impact.bodies.includes(pallet)
      ) {
        this.impacts.push({ time: impact.time, speed: impact.speed, energy: impact.energy });
      }
    }
    const frame = this.frame();
    if (this.impacts.length > 0) {
      this.furthestFront = Math.max(this.furthestFront, frame.front);
      this.rebound = Math.max(this.rebound, this.furthestFront - frame.front);
    }
    return { ...frame, rebound: this.rebound };
  }

  /** Runs for `seconds` of simulated time; `observe` sees every step. */
  run(seconds: number, observe?: (frame: BenchFrame) => void): BenchFrame {
    let frame = this.frame();
    const steps = Math.round(seconds / this.dt);
    for (let i = 0; i < steps; i++) {
      frame = this.step();
      observe?.(frame);
    }
    return frame;
  }

  frame(): BenchFrame {
    const handle = this.bodies.pallets[0];
    if (handle === undefined) throw new Error('The bench has no pallet');
    const palletPose = this.world.getPose(handle);
    const velocity = this.world.getVelocity(handle);
    const { x, y, z, w } = palletPose.rotation;
    // Front edge: the farthest point of the footprint along X (body X and Z axes in world X).
    const { length, width } = this.layout.mtbh;
    const axisXx = 1 - 2 * (y * y + z * z);
    const axisZx = 2 * (x * z + y * w);
    const front =
      palletPose.position.x + (length / 2) * Math.abs(axisXx) + (width / 2) * Math.abs(axisZx);
    const heading = Math.atan2(-2 * (x * z - w * y), axisXx);
    const rollerSpeed = this.rollerSpeedAt(this.time);
    const platform = this.bodies.platforms[0];
    const platformRotation =
      platform === undefined ? undefined : this.world.getPose(platform).rotation;
    return {
      time: this.time,
      rollerSpeed,
      rollerAngle: this.rollerAngle,
      pallet: { pose: palletPose, velocity },
      front,
      yaw: wrapAngle(heading - this.alongYaw()),
      slip: rollerSpeed - velocity.linear.x,
      tilt: 2 * Math.acos(Math.min(1, Math.abs(w))),
      platformAngle:
        platformRotation === undefined ? 0 : 2 * Math.atan2(platformRotation.y, platformRotation.w),
      impacts: [...this.impacts],
      rebound: this.rebound,
      fallen: palletPose.position.y < BENCH_SURFACE_HEIGHT - 0.1,
    };
  }

  stateHash(): string {
    return this.world.stateHash();
  }

  dispose(): void {
    this.world.dispose();
  }

  /** Trapezoidal start-up from rest: v(t) = min(v, a·t) (T04). */
  private rollerSpeedAt(time: number): number {
    return Math.min(num(this.params, 'speed', 0.25), num(this.params, 'accel', 0.5) * time);
  }

  /** Angular speed of the turntable: one trapezoidal turn starting at `turnStart` (T04 §2). */
  private platformSpeedAt(time: number): number {
    const t = time - num(this.params, 'turnStart', 0.5);
    if (this.bodies.platforms.length === 0 || t <= 0) return 0;
    const angle = num(this.params, 'turnAngle', Math.PI / 2);
    const speed = num(this.params, 'maxAngularSpeed', Math.PI / 12);
    const accel = num(this.params, 'angularAccel', Math.PI / 6);
    const cruise = angle >= (speed * speed) / accel;
    const total = cruise ? angle / speed + speed / accel : 2 * Math.sqrt(angle / accel);
    if (t >= total) return 0;
    return Math.min(accel * t, cruise ? speed : Infinity, accel * (total - t));
  }

  /** Yaw at which the pallet's chosen side runs along travel. */
  private alongYaw(): number {
    return this.params.palletSideAlongTravel === 'short' ? Math.PI / 2 : 0;
  }
}

export function benchDefaults(): ParamValues {
  const values: ParamValues = {};
  for (const p of BENCH_PARAMETERS) if (p.default !== undefined) values[p.key] = p.default;
  return values;
}

function benchLayout(params: ParamValues, mtbh: Mtbh): BenchLayout {
  const length = num(params, 'conveyorLength', 3);
  const deck = benchDeck(params);
  const turntable = params.turntable === true;
  const type = MTBH_TYPES[mtbhType(params)];
  const short = params.palletSideAlongTravel === 'short';
  const halfAlong = (short ? mtbh.width : mtbh.length) / 2;
  const laneCount = Math.max(1, Math.round(num(params, 'lanes', 1)));
  const layout: BenchLayout = {
    conveyorLength: length,
    surfaceHeight: BENCH_SURFACE_HEIGHT,
    usableWidth: BENCH_USABLE_WIDTH,
    deck,
    rollers: deckRollers(params, deck, length, turntable),
    strands:
      deck === 'rollers' ? [] : strandPositions(params).map((z) => ({ z, width: STRAND_WIDTH })),
    guides:
      params.guidesEnabled === true ? guideBoxes(params, short ? mtbh.length : mtbh.width) : [],
    laneOffsets: Array.from({ length: laneCount }, (_, i) => i * LANE_SPACING),
    mtbh: {
      visualAsset: 'load' in type ? type.load : type.pallet,
      placement: mtbh.placement,
      length: mtbh.length,
      width: mtbh.width,
      height: mtbh.height,
      mass: mtbh.massProperties.mass,
      centerOfMass: mtbh.massProperties.centerOfMass,
      start: {
        x: turntable ? length / 2 : START_GAP + halfAlong,
        z: num(params, 'startOffsetZ', 0),
        yaw: (short ? Math.PI / 2 : 0) + num(params, 'startYaw', 0),
      },
    },
  };
  if (turntable) {
    layout.platform = {
      center: {
        x: length / 2,
        y: BENCH_SURFACE_HEIGHT - num(params, 'rollerDiameter', 0.089) / 2,
        z: 0,
      },
    };
  }
  if (params.stopEnabled !== true) return layout;
  const height = num(params, 'stopHeight', 0.1);
  return {
    ...layout,
    stop: {
      center: {
        x: num(params, 'stopPosition', 2.8) + STOP_THICKNESS / 2,
        y: BENCH_SURFACE_HEIGHT + (height - STOP_DEPTH) / 2,
        z: 0,
      },
      halfExtents: {
        x: STOP_THICKNESS / 2,
        y: (height + STOP_DEPTH) / 2,
        z: num(params, 'stopWidth', 0.1) / 2,
      },
      material: String(params.stopMaterial),
    },
  };
}

/**
 * Rollers of lane 0: full-width rollers at the roller pitch (centred on the platform for a
 * turntable), or the short plan-A rollers under each chain strand.
 */
function deckRollers(
  params: ParamValues,
  deck: BenchDeck,
  length: number,
  turntable: boolean,
): BenchLayout['rollers'] {
  if (deck === 'chains') return [];
  if (deck === 'chainsPlanA') {
    const count = Math.floor(length / PLAN_A_PITCH + 1e-9);
    return strandPositions(params).flatMap((z) =>
      Array.from({ length: count }, (_, i) => ({
        x: PLAN_A_PITCH / 2 + i * PLAN_A_PITCH,
        y: BENCH_SURFACE_HEIGHT - PLAN_A_DIAMETER / 2,
        z,
        diameter: PLAN_A_DIAMETER,
        length: STRAND_WIDTH,
      })),
    );
  }
  const pitch = num(params, 'rollerPitch', 0.2063);
  const diameter = num(params, 'rollerDiameter', 0.089);
  const count = Math.floor(length / pitch + 1e-9);
  // On a turntable the rollers are symmetric about the platform centre, so it turns about its middle.
  const first = turntable ? (length - (count - 1) * pitch) / 2 : pitch / 2;
  return Array.from({ length: count }, (_, i) => ({
    x: first + i * pitch,
    y: BENCH_SURFACE_HEIGHT - diameter / 2,
    z: 0,
    diameter,
    length: BENCH_USABLE_WIDTH,
  }));
}

/** Strand axes spread evenly over the track gauge, outer strands at ±gauge / 2. */
function strandPositions(params: ParamValues): number[] {
  const count = Math.max(2, Math.round(num(params, 'chainCount', 3)));
  const gauge = num(params, 'chainGauge', 1.05);
  return Array.from({ length: count }, (_, i) => (i / (count - 1) - 0.5) * gauge);
}

/**
 * Two rails from `guideStart` to the conveyor end with their inner faces at half the pallet width
 * across plus the clearance, and a funnel plate before each rail whose mouth is wider by
 * funnelLength · tan(funnelAngle) (S08).
 */
function guideBoxes(params: ParamValues, palletAcross: number): BenchBox[] {
  const start = num(params, 'guideStart', 2);
  const end = num(params, 'conveyorLength', 3);
  const inner = palletAcross / 2 + num(params, 'guideClearance', 0.025);
  const height = num(params, 'guideHeight', 0.15);
  const angle = num(params, 'funnelAngle', Math.PI / 18);
  const funnel = num(params, 'funnelLength', 0.3);
  const material = String(params.guideMaterial);
  const y = BENCH_SURFACE_HEIGHT + (GUIDE_GAP + height) / 2;
  const halfY = (height - GUIDE_GAP) / 2;
  const t = GUIDE_THICKNESS / 2;
  return [1, -1].flatMap((side) => [
    {
      center: { x: (start + end) / 2, y, z: side * (inner + t) },
      halfExtents: { x: (end - start) / 2, y: halfY, z: t },
      yaw: 0,
      material,
    },
    {
      // Inner face from (start − funnel, inner + funnel·tan) to (start, inner); the centre lies
      // half a thickness outwards along the face normal (sin, cos) of the +Z plate.
      center: {
        x: start - funnel / 2 + t * Math.sin(angle),
        y,
        z: side * (inner + (funnel * Math.tan(angle)) / 2 + t * Math.cos(angle)),
      },
      halfExtents: { x: funnel / Math.cos(angle) / 2, y: halfY, z: t },
      yaw: side * angle,
      material,
    },
  ]);
}

/** Adds the conveyor, guides and stop of one lane, shifted `laneZ` across. */
function addLane(
  world: PhysicsBackend,
  layout: BenchLayout,
  params: ParamValues,
  laneZ: number,
  bodies: BenchBodies,
): void {
  const rollerMaterial = String(params.rollerMaterial);
  const rollerFriction =
    params.rollerMuOverride === true ? { friction: num(params, 'rollerMu', 0.35) } : {};
  const center = layout.platform?.center;
  if (center !== undefined) {
    // Turntable: the locked rollers form one kinematic body turning about the platform centre.
    const shapes: Shape[] = layout.rollers.map((roller) => ({
      kind: 'cylinder',
      radius: roller.diameter / 2,
      halfLength: roller.length / 2,
      axis: 'z',
      offset: { x: roller.x - center.x, y: 0, z: roller.z },
    }));
    bodies.platforms.push(
      world.addBody({
        motion: 'kinematic',
        pose: pose({ ...center, z: center.z + laneZ }),
        shapes,
        material: rollerMaterial,
        ...rollerFriction,
      }),
    );
  } else {
    const target = layout.deck === 'chainsPlanA' ? bodies.smallRollers : bodies.rollers;
    for (const roller of layout.rollers) {
      target.push(
        world.addBody({
          motion: 'kinematic',
          pose: pose({ x: roller.x, y: roller.y, z: roller.z + laneZ }),
          shapes: [
            {
              kind: 'cylinder',
              radius: roller.diameter / 2,
              halfLength: roller.length / 2,
              axis: 'z',
              offset: ZERO,
            },
          ],
          material: layout.deck === 'rollers' ? rollerMaterial : 'steel-chain',
          ...(layout.deck === 'rollers' ? rollerFriction : {}),
        }),
      );
    }
  }
  if (layout.deck === 'chains') {
    for (const strand of layout.strands) {
      bodies.strands.push(
        world.addBody({
          motion: 'static',
          pose: pose({
            x: layout.conveyorLength / 2,
            y: BENCH_SURFACE_HEIGHT - STRAND_HEIGHT / 2,
            z: strand.z + laneZ,
          }),
          shapes: [
            {
              kind: 'box',
              halfExtents: {
                x: layout.conveyorLength / 2,
                y: STRAND_HEIGHT / 2,
                z: strand.width / 2,
              },
              offset: ZERO,
            },
          ],
          material: 'steel-chain',
        }),
      );
    }
  }
  for (const guide of layout.guides) {
    world.addBody({
      motion: 'static',
      pose: {
        position: { ...guide.center, z: guide.center.z + laneZ },
        rotation: yawRotation(guide.yaw),
      },
      shapes: [{ kind: 'box', halfExtents: guide.halfExtents, offset: ZERO }],
      material: guide.material,
    });
  }
  const stop = layout.stop;
  if (stop !== undefined) {
    bodies.stops.push(
      world.addBody({
        motion: 'static',
        pose: pose({ ...stop.center, z: stop.center.z + laneZ }),
        shapes: [{ kind: 'box', halfExtents: stop.halfExtents, offset: ZERO }],
        material: stop.material,
        ...(params.stopRestitutionOverride === true
          ? { restitution: num(params, 'stopRestitution', 0.1) }
          : {}),
      }),
    );
  }
}

function addPallet(
  world: PhysicsBackend,
  layout: BenchLayout,
  mtbh: Mtbh,
  laneZ: number,
): BodyHandle {
  const { start } = layout.mtbh;
  return world.addBody({
    motion: 'dynamic',
    pose: {
      position: { x: start.x, y: BENCH_SURFACE_HEIGHT + 0.001, z: start.z + laneZ },
      rotation: yawRotation(start.yaw),
    },
    shapes: mtbh.shapes,
    material: mtbh.material,
    massProperties: mtbh.massProperties,
    ccd: true,
  });
}

function buildBenchMtbh(params: ParamValues): Mtbh {
  const type = MTBH_TYPES[mtbhType(params)];
  const palletAsset = builtInCatalog.get(type.pallet);
  const palletType = PALLET_TYPES.find((p) => p.id === type.palletType);
  if (palletAsset.physics === undefined || palletType === undefined) {
    throw new Error(
      `Pallet model ${type.pallet} has no rigid-body data or type ${type.palletType} is unknown`,
    );
  }
  const pallet = {
    ...palletAsset.physics,
    bounds: palletAsset.bounds,
    construction: palletConstruction(palletType),
    material: type.material,
  };
  if (!('load' in type)) return buildMtbh({ pallet });
  return buildMtbh({
    pallet,
    load: {
      mass: num(params, 'loadMass', 600),
      bounds: builtInCatalog.get(type.load).bounds,
      centerOffset: {
        x: num(params, 'comOffsetX', 0),
        y: num(params, 'comOffsetY', 0),
        z: num(params, 'comOffsetZ', 0),
      },
    },
  });
}

function mtbhType(params: ParamValues): MtbhType {
  const value = String(params.mtbhType);
  if (!(value in MTBH_TYPES)) throw new Error(`Unknown pallet and load "${value}"`);
  return value as MtbhType;
}

function benchDeck(params: ParamValues): BenchDeck {
  const value = params.deck;
  if (value === 'rollers' || value === 'chains' || value === 'chainsPlanA') return value;
  throw new Error(`Unknown deck "${String(value)}"`);
}

function pose(position: Vec3): Pose {
  return { position, rotation: yawRotation(0) };
}

/** Rotation about +Y by `yaw` (right-handed: +X turns toward −Z). */
function yawRotation(yaw: number): Pose['rotation'] {
  return { x: 0, y: Math.sin(yaw / 2), z: 0, w: Math.cos(yaw / 2) };
}

/** Angle in (−π, π]. */
function wrapAngle(angle: number): number {
  const wrapped = angle - 2 * Math.PI * Math.floor((angle + Math.PI) / (2 * Math.PI));
  return wrapped === -Math.PI ? Math.PI : wrapped;
}
