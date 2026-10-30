// Physics test bench: a roller section with an optional fixed stop and one MTBH built from real models,
// run with the Jolt backend at the fixed step. Shared by the reference scenarios P1–P7 and the editor.
// Bench frame: X along travel from the conveyor start, Y up, Z across; the floor is at y = 0.
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
const FLOOR_HALF = { x: 10, y: 0.05, z: 10 };
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

/** Static description of the bench, for drawing it. */
export interface BenchLayout {
  conveyorLength: number;
  surfaceHeight: number;
  usableWidth: number;
  /** Roller axes run along Z. */
  rollers: { x: number; y: number; diameter: number }[];
  stop?: { center: Vec3; halfExtents: Vec3; material: string };
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
  };
}

export interface BenchImpact {
  time: number;
  speed: number;
  energy: number;
}

/** State of the bench after a step. */
export interface BenchFrame {
  time: number;
  rollerSpeed: number;
  /** Accumulated roller rotation (rad), for drawing. */
  rollerAngle: number;
  pallet: { pose: Pose; velocity: Velocity };
  /** X of the pallet's front edge (m). */
  front: number;
  /** Roller surface speed minus pallet speed along travel (m/s). */
  slip: number;
  /** Angle between the pallet's up axis and the vertical (rad). */
  tilt: number;
  /** Pallet impacts against the stop so far. */
  impacts: BenchImpact[];
  /** Largest backward travel after the first stop impact (m). */
  rebound: number;
  /** The pallet has left the conveyor and dropped. */
  fallen: boolean;
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
    private readonly rollers: BodyHandle[],
    private readonly pallet: BodyHandle,
    private readonly stop: BodyHandle | undefined,
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
    const rollers = layout.rollers.map((roller) =>
      world.addBody({
        motion: 'kinematic',
        pose: pose({ x: roller.x, y: roller.y, z: 0 }),
        shapes: [
          {
            kind: 'cylinder',
            radius: roller.diameter / 2,
            halfLength: BENCH_USABLE_WIDTH / 2,
            axis: 'z',
            offset: ZERO,
          },
        ],
        material: String(params.rollerMaterial),
        ...(params.rollerMuOverride === true ? { friction: num(params, 'rollerMu', 0.35) } : {}),
      }),
    );
    const stop =
      layout.stop === undefined
        ? undefined
        : world.addBody({
            motion: 'static',
            pose: pose(layout.stop.center),
            shapes: [{ kind: 'box', halfExtents: layout.stop.halfExtents, offset: ZERO }],
            material: layout.stop.material,
            ...(params.stopRestitutionOverride === true
              ? { restitution: num(params, 'stopRestitution', 0.1) }
              : {}),
          });

    const pallet = world.addBody({
      motion: 'dynamic',
      pose: pose({ x: START_GAP + mtbh.length / 2, y: BENCH_SURFACE_HEIGHT + 0.001, z: 0 }),
      shapes: mtbh.shapes,
      material: mtbh.material,
      massProperties: mtbh.massProperties,
      ccd: true,
    });
    if (params.startAtSpeed === true)
      world.setVelocity(pallet, { x: num(params, 'speed', 0.25), y: 0, z: 0 }, ZERO);
    return new PhysicsBench(layout, params, world, rollers, pallet, stop);
  }

  /** Advances one fixed step and returns the new state. */
  step(): BenchFrame {
    const speed = this.rollerSpeedAt(this.time);
    const omega = rollerAngularVelocity(speed, num(this.params, 'rollerDiameter', 0.089));
    for (const roller of this.rollers)
      this.world.setVelocity(roller, ZERO, { x: 0, y: 0, z: -omega });
    this.world.step(this.dt);
    this.time += this.dt;
    this.rollerAngle += omega * this.dt;

    for (const impact of this.world.takeImpacts()) {
      if (
        this.stop !== undefined &&
        impact.bodies.includes(this.stop) &&
        impact.bodies.includes(this.pallet)
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
    const palletPose = this.world.getPose(this.pallet);
    const velocity = this.world.getVelocity(this.pallet);
    const q = palletPose.rotation;
    // Front edge: the body point (length / 2, 0, 0) in world coordinates.
    const halfLength = this.layout.mtbh.length / 2;
    const front = palletPose.position.x + halfLength * (1 - 2 * (q.y * q.y + q.z * q.z));
    const rollerSpeed = this.rollerSpeedAt(this.time);
    return {
      time: this.time,
      rollerSpeed,
      rollerAngle: this.rollerAngle,
      pallet: { pose: palletPose, velocity },
      front,
      slip: rollerSpeed - velocity.linear.x,
      tilt: 2 * Math.acos(Math.min(1, Math.abs(q.w))),
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
}

export function benchDefaults(): ParamValues {
  const values: ParamValues = {};
  for (const p of BENCH_PARAMETERS) if (p.default !== undefined) values[p.key] = p.default;
  return values;
}

function benchLayout(params: ParamValues, mtbh: Mtbh): BenchLayout {
  const length = num(params, 'conveyorLength', 3);
  const pitch = num(params, 'rollerPitch', 0.2063);
  const diameter = num(params, 'rollerDiameter', 0.089);
  const count = Math.floor(length / pitch + 1e-9);
  const rollers = Array.from({ length: count }, (_, i) => ({
    x: pitch / 2 + i * pitch,
    y: BENCH_SURFACE_HEIGHT - diameter / 2,
    diameter,
  }));
  const type = MTBH_TYPES[mtbhType(params)];
  const layout: BenchLayout = {
    conveyorLength: length,
    surfaceHeight: BENCH_SURFACE_HEIGHT,
    usableWidth: BENCH_USABLE_WIDTH,
    rollers,
    mtbh: {
      visualAsset: 'load' in type ? type.load : type.pallet,
      placement: mtbh.placement,
      length: mtbh.length,
      width: mtbh.width,
      height: mtbh.height,
      mass: mtbh.massProperties.mass,
      centerOfMass: mtbh.massProperties.centerOfMass,
    },
  };
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

function pose(position: Vec3): Pose {
  return { position, rotation: { x: 0, y: 0, z: 0, w: 1 } };
}
