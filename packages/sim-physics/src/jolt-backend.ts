// Jolt adapter (ADR-0002): the only module that imports the engine. It lowers the engine defaults that
// break the ±5 mm stop criterion (T15 §3), gives every contact the exact friction and restitution of
// its material pair (S16), adds conveyor surface velocities (plan B) and records impacts.
import { GRAVITY, type Vec3 } from '@intracraft/core-model';
import Jolt from 'jolt-physics';
import type {
  BodyDesc,
  BodyHandle,
  ContactPairLookup,
  Impact,
  PhysicsBackend,
  Pose,
  RayHit,
  Shape,
  Velocity,
} from './backend';

/** Engine settings that differ from Jolt's defaults (defaults measured in T15 §3). */
export const JOLT_SETTINGS = {
  /** Tolerated penetration (m); the default 0.02 m exceeds the ±5 mm stop tolerance. */
  penetrationSlop: 0.001,
  /** Distance at which contacts are created ahead of time (m); default 0.02 m. */
  speculativeContactDistance: 0.005,
  /** Approach speed below which restitution is ignored (m/s); the default 1 m/s hides stop rebounds. */
  minVelocityForRestitution: 0.01,
  /** Air drag on dynamic bodies (1/s); the default 0.05 slows a pallet by 0.0125 m/s² at 0.25 m/s. */
  linearDamping: 0,
  angularDamping: 0,
} as const;

/** New contacts approaching slower than this are not reported as impacts (m/s). */
export const IMPACT_MIN_SPEED = 0.01;

/** Rounding of box and cylinder edges (m), capped at half the thinnest side. */
const CONVEX_RADIUS = 0.002;
const LAYER_STATIC = 0;
const LAYER_MOVING = 1;
const ZERO: Vec3 = { x: 0, y: 0, z: 0 };
const HALF_SQRT2 = Math.SQRT1_2;
/** Jolt cylinders run along Y; these rotations turn them onto the requested axis. */
const CYLINDER_ROTATION = {
  x: [0, 0, -HALF_SQRT2, HALF_SQRT2],
  y: [0, 0, 0, 1],
  z: [HALF_SQRT2, 0, 0, HALF_SQRT2],
} as const;

type JoltApi = typeof Jolt;

interface BodyRecord {
  handle: BodyHandle;
  body: Jolt.Body;
  id: Jolt.BodyID;
  motion: BodyDesc['motion'];
  material: string;
  /** kg; Infinity for static and kinematic bodies. */
  mass: number;
  friction?: number;
  restitution?: number;
  surface?: Vec3;
}

export interface JoltBackendOptions {
  contactPair: ContactPairLookup;
}

let joltModule: Promise<JoltApi> | undefined;

/** Loads the engine once (WASM) and creates an empty world with gravity −Y. */
export async function createJoltBackend(options: JoltBackendOptions): Promise<PhysicsBackend> {
  joltModule ??= Jolt();
  return new JoltBackend(await joltModule, options.contactPair);
}

class JoltBackend implements PhysicsBackend {
  private readonly joltInterface: Jolt.JoltInterface;
  private readonly bodies: Jolt.BodyInterface;
  private readonly listener: Jolt.ContactListenerJS;
  private readonly records = new Map<BodyHandle, BodyRecord>();
  private readonly byPointer = new Map<number, BodyRecord>();
  private readonly byBodyId = new Map<number, BodyRecord>();
  private readonly stepImpacts = new Map<string, Omit<Impact, 'time'>>();
  private impacts: Impact[] = [];
  /** Errors thrown inside engine callbacks are kept and rethrown after the step. */
  private callbackError: unknown;
  private nextHandle = 1;
  private time = 0;
  /** Vertical speed gravity adds in one step, g·dt (m/s). */
  private gravityStep = 0;
  // Reused engine vectors: avoid allocating inside the step loop.
  private readonly vecA: Jolt.Vec3;
  private readonly vecB: Jolt.Vec3;

  constructor(
    private readonly jolt: JoltApi,
    private readonly contactPair: ContactPairLookup,
  ) {
    const settings = new jolt.JoltSettings();
    const objectFilter = new jolt.ObjectLayerPairFilterTable(2);
    objectFilter.EnableCollision(LAYER_STATIC, LAYER_MOVING);
    objectFilter.EnableCollision(LAYER_MOVING, LAYER_MOVING);
    const broadPhase = new jolt.BroadPhaseLayerInterfaceTable(2, 2);
    broadPhase.MapObjectToBroadPhaseLayer(LAYER_STATIC, new jolt.BroadPhaseLayer(0));
    broadPhase.MapObjectToBroadPhaseLayer(LAYER_MOVING, new jolt.BroadPhaseLayer(1));
    settings.mObjectLayerPairFilter = objectFilter;
    settings.mBroadPhaseLayerInterface = broadPhase;
    settings.mObjectVsBroadPhaseLayerFilter = new jolt.ObjectVsBroadPhaseLayerFilterTable(
      broadPhase,
      2,
      objectFilter,
      2,
    );
    this.joltInterface = new jolt.JoltInterface(settings);
    jolt.destroy(settings);

    const system = this.joltInterface.GetPhysicsSystem();
    const physics = system.GetPhysicsSettings();
    physics.mPenetrationSlop = JOLT_SETTINGS.penetrationSlop;
    physics.mSpeculativeContactDistance = JOLT_SETTINGS.speculativeContactDistance;
    physics.mMinVelocityForRestitution = JOLT_SETTINGS.minVelocityForRestitution;
    system.SetPhysicsSettings(physics);
    this.vecA = new jolt.Vec3(0, -GRAVITY, 0);
    this.vecB = new jolt.Vec3(0, 0, 0);
    system.SetGravity(this.vecA);
    this.bodies = system.GetBodyInterface();

    this.listener = new jolt.ContactListenerJS();
    this.listener.OnContactValidate = () => jolt.ValidateResult_AcceptAllContactsForThisBodyPair;
    this.listener.OnContactAdded = (body1, body2, manifold, contactSettings) => {
      this.guard(() => {
        this.configureContact(body1, body2, contactSettings);
        this.recordImpact(body1, body2, manifold);
      });
    };
    this.listener.OnContactPersisted = (body1, body2, _manifold, contactSettings) => {
      this.guard(() => this.configureContact(body1, body2, contactSettings));
    };
    this.listener.OnContactRemoved = () => {};
    system.SetContactListener(this.listener);
  }

  addBody(desc: BodyDesc): BodyHandle {
    const jolt = this.jolt;
    if (desc.motion === 'dynamic' && desc.massProperties === undefined) {
      throw new Error('A dynamic body needs mass properties (mass, centre of mass, inertia)');
    }
    const { position: p, rotation: r } = desc.pose;
    const position = new jolt.RVec3(p.x, p.y, p.z);
    const rotation = new jolt.Quat(r.x, r.y, r.z, r.w);
    const motionType = {
      static: jolt.EMotionType_Static,
      kinematic: jolt.EMotionType_Kinematic,
      dynamic: jolt.EMotionType_Dynamic,
    }[desc.motion];
    const layer = desc.motion === 'static' ? LAYER_STATIC : LAYER_MOVING;
    const settings = new jolt.BodyCreationSettings(
      this.createShape(desc),
      position,
      rotation,
      motionType,
      layer,
    );
    const mass = desc.massProperties;
    if (mass !== undefined) {
      settings.mOverrideMassProperties = jolt.EOverrideMassProperties_MassAndInertiaProvided;
      settings.mMassPropertiesOverride.mMass = mass.mass;
      this.vecB.Set(mass.inertia.x, mass.inertia.y, mass.inertia.z);
      settings.mMassPropertiesOverride.mInertia = jolt.Mat44.prototype.sScaleVec3(this.vecB);
      settings.mLinearDamping = JOLT_SETTINGS.linearDamping;
      settings.mAngularDamping = JOLT_SETTINGS.angularDamping;
      // Sleeping is off until conveyors wake the pallets they touch (A07 §4).
      settings.mAllowSleeping = false;
    }
    if (desc.ccd === true) settings.mMotionQuality = jolt.EMotionQuality_LinearCast;

    const body = this.bodies.CreateBody(settings);
    jolt.destroy(settings);
    jolt.destroy(position);
    jolt.destroy(rotation);
    const id = body.GetID();
    this.bodies.AddBody(id, jolt.EActivation_Activate);

    const record: BodyRecord = {
      handle: this.nextHandle++,
      body,
      id,
      motion: desc.motion,
      material: desc.material,
      mass: mass?.mass ?? Infinity,
      ...(desc.friction === undefined ? {} : { friction: desc.friction }),
      ...(desc.restitution === undefined ? {} : { restitution: desc.restitution }),
    };
    this.records.set(record.handle, record);
    this.byPointer.set(jolt.getPointer(body), record);
    this.byBodyId.set(id.GetIndexAndSequenceNumber(), record);
    return record.handle;
  }

  removeBody(handle: BodyHandle): void {
    const record = this.record(handle);
    this.records.delete(handle);
    this.byPointer.delete(this.jolt.getPointer(record.body));
    this.byBodyId.delete(record.id.GetIndexAndSequenceNumber());
    this.bodies.RemoveBody(record.id);
    this.bodies.DestroyBody(record.id);
  }

  setVelocity(handle: BodyHandle, linear: Vec3, angular: Vec3): void {
    this.vecA.Set(linear.x, linear.y, linear.z);
    this.vecB.Set(angular.x, angular.y, angular.z);
    this.bodies.SetLinearAndAngularVelocity(this.record(handle).id, this.vecA, this.vecB);
  }

  setSurfaceVelocity(handle: BodyHandle, velocity: Vec3): boolean {
    this.record(handle).surface = { ...velocity };
    return true;
  }

  getPose(handle: BodyHandle): Pose {
    const body = this.record(handle).body;
    const p = body.GetPosition();
    const position = { x: p.GetX(), y: p.GetY(), z: p.GetZ() };
    const q = body.GetRotation();
    return { position, rotation: { x: q.GetX(), y: q.GetY(), z: q.GetZ(), w: q.GetW() } };
  }

  getVelocity(handle: BodyHandle): Velocity {
    const body = this.record(handle).body;
    const v = body.GetLinearVelocity();
    const linear = { x: v.GetX(), y: v.GetY(), z: v.GetZ() };
    const w = body.GetAngularVelocity();
    return { linear, angular: { x: w.GetX(), y: w.GetY(), z: w.GetZ() } };
  }

  castRay(
    origin: Vec3,
    direction: Vec3,
    maxDistance: number,
    accept?: (body: BodyHandle) => boolean,
  ): RayHit | undefined {
    const jolt = this.jolt;
    const length = Math.hypot(direction.x, direction.y, direction.z);
    const scale = maxDistance / length;
    const start = new jolt.RVec3(origin.x, origin.y, origin.z);
    const span = new jolt.Vec3(direction.x * scale, direction.y * scale, direction.z * scale);
    const ray = new jolt.RRayCast(start, span);
    const raySettings = new jolt.RayCastSettings();
    const collector = new jolt.CastRayAllHitCollisionCollector();
    const broadPhaseFilter = new jolt.DefaultBroadPhaseLayerFilter(
      this.joltInterface.GetObjectVsBroadPhaseLayerFilter(),
      LAYER_MOVING,
    );
    const objectFilter = new jolt.DefaultObjectLayerFilter(
      this.joltInterface.GetObjectLayerPairFilter(),
      LAYER_MOVING,
    );
    const bodyFilter = new jolt.BodyFilter();
    const shapeFilter = new jolt.ShapeFilter();
    this.joltInterface
      .GetPhysicsSystem()
      .GetNarrowPhaseQuery()
      .CastRay(
        ray,
        raySettings,
        collector,
        broadPhaseFilter,
        objectFilter,
        bodyFilter,
        shapeFilter,
      );
    // All hits, nearest first: the first accepted one is the answer.
    collector.Sort();
    let hit: { record: BodyRecord; fraction: number } | undefined;
    for (let i = 0; i < collector.mHits.size(); i++) {
      const result = collector.mHits.at(i);
      const record = this.byBodyId.get(result.mBodyID.GetIndexAndSequenceNumber());
      if (record === undefined || (accept !== undefined && !accept(record.handle))) continue;
      hit = { record, fraction: result.mFraction };
      break;
    }
    for (const object of [
      start,
      span,
      ray,
      raySettings,
      collector,
      broadPhaseFilter,
      objectFilter,
      bodyFilter,
      shapeFilter,
    ]) {
      jolt.destroy(object);
    }
    if (hit === undefined) return undefined;
    return { body: hit.record.handle, distance: hit.fraction * maxDistance };
  }

  step(dt: number): void {
    this.gravityStep = GRAVITY * dt;
    this.stepImpacts.clear();
    this.joltInterface.Step(dt, 1);
    this.time += dt;
    if (this.callbackError !== undefined) {
      const error = this.callbackError;
      this.callbackError = undefined;
      throw error;
    }
    for (const impact of this.stepImpacts.values())
      this.impacts.push({ ...impact, time: this.time });
  }

  takeImpacts(): Impact[] {
    const impacts = this.impacts;
    this.impacts = [];
    return impacts;
  }

  stateHash(): string {
    const values: number[] = [];
    for (const record of this.records.values()) {
      if (record.motion === 'static') continue;
      const { position: p, rotation: q } = this.getPose(record.handle);
      const { linear: v, angular: w } = this.getVelocity(record.handle);
      values.push(p.x, p.y, p.z, q.x, q.y, q.z, q.w, v.x, v.y, v.z, w.x, w.y, w.z);
    }
    return fnv1a(new Uint8Array(Float64Array.from(values).buffer));
  }

  dispose(): void {
    this.jolt.destroy(this.joltInterface);
    this.jolt.destroy(this.listener);
    this.jolt.destroy(this.vecA);
    this.jolt.destroy(this.vecB);
    this.records.clear();
  }

  private record(handle: BodyHandle): BodyRecord {
    const record = this.records.get(handle);
    if (record === undefined) throw new Error(`Unknown body ${handle}`);
    return record;
  }

  /** Engine shape for the body: one shape or a compound, with the centre of mass moved where requested. */
  private createShape(desc: BodyDesc): Jolt.Shape {
    const jolt = this.jolt;
    let settings: Jolt.ShapeSettings;
    if (desc.shapes.length === 1 && desc.shapes[0] !== undefined) {
      settings = this.placedShape(desc.shapes[0]);
    } else {
      const compound = new jolt.StaticCompoundShapeSettings();
      for (const shape of desc.shapes) {
        const [x, y, z, w] =
          shape.kind === 'cylinder' ? CYLINDER_ROTATION[shape.axis] : CYLINDER_ROTATION.y;
        const offset = new jolt.Vec3(shape.offset.x, shape.offset.y, shape.offset.z);
        const rotation = new jolt.Quat(x, y, z, w);
        compound.AddShape(offset, rotation, this.baseShape(shape), 0);
        jolt.destroy(offset);
        jolt.destroy(rotation);
      }
      settings = compound;
    }
    const center = desc.massProperties?.centerOfMass;
    if (center !== undefined) {
      const natural = createdShape(settings).GetCenterOfMass();
      const offset = new jolt.Vec3(
        center.x - natural.GetX(),
        center.y - natural.GetY(),
        center.z - natural.GetZ(),
      );
      settings = new jolt.OffsetCenterOfMassShapeSettings(offset, settings);
      jolt.destroy(offset);
    }
    return createdShape(settings);
  }

  private placedShape(shape: Shape): Jolt.ShapeSettings {
    const base = this.baseShape(shape);
    const [x, y, z, w] =
      shape.kind === 'cylinder' ? CYLINDER_ROTATION[shape.axis] : CYLINDER_ROTATION.y;
    const { offset } = shape;
    if (w === 1 && offset.x === 0 && offset.y === 0 && offset.z === 0) return base;
    const position = new this.jolt.Vec3(offset.x, offset.y, offset.z);
    const rotation = new this.jolt.Quat(x, y, z, w);
    const placed = new this.jolt.RotatedTranslatedShapeSettings(position, rotation, base);
    this.jolt.destroy(position);
    this.jolt.destroy(rotation);
    return placed;
  }

  private baseShape(shape: Shape): Jolt.ShapeSettings {
    if (shape.kind === 'box') {
      const h = shape.halfExtents;
      const halfExtents = new this.jolt.Vec3(h.x, h.y, h.z);
      const settings = new this.jolt.BoxShapeSettings(halfExtents, convexRadius(h.x, h.y, h.z));
      this.jolt.destroy(halfExtents);
      return settings;
    }
    return new this.jolt.CylinderShapeSettings(
      shape.halfLength,
      shape.radius,
      convexRadius(shape.halfLength, shape.radius),
    );
  }

  /** Exact pair friction and restitution; per-instance values of the static/kinematic body win. */
  private configureContact(pointer1: number, pointer2: number, settingsPointer: number): void {
    const r1 = this.byPointer.get(pointer1);
    const r2 = this.byPointer.get(pointer2);
    if (r1 === undefined || r2 === undefined) return;
    const [surface, other] = r1.motion === 'dynamic' ? [r2, r1] : [r1, r2];
    const friction = surface.friction ?? other.friction;
    const restitution = surface.restitution ?? other.restitution;
    const pair =
      friction === undefined || restitution === undefined
        ? this.contactPair(r1.material, r2.material)
        : undefined;
    const settings = this.jolt.wrapPointer(settingsPointer, this.jolt.ContactSettings);
    settings.mCombinedFriction = friction ?? pair?.mu ?? 0;
    settings.mCombinedRestitution = restitution ?? pair?.restitution ?? 0;
    if (r1.surface !== undefined || r2.surface !== undefined) {
      // Jolt expects the surface velocity of body 2 relative to body 1, in world space.
      const s1 = r1.surface ?? ZERO;
      const s2 = r2.surface ?? ZERO;
      this.vecA.Set(s2.x - s1.x, s2.y - s1.y, s2.z - s1.z);
      settings.mRelativeLinearSurfaceVelocity = this.vecA;
    }
  }

  /** Keeps the fastest approach per body pair in this step. */
  private recordImpact(pointer1: number, pointer2: number, manifoldPointer: number): void {
    const r1 = this.byPointer.get(pointer1);
    const r2 = this.byPointer.get(pointer2);
    if (r1 === undefined || r2 === undefined) return;
    const manifold = this.jolt.wrapPointer(manifoldPointer, this.jolt.ContactManifold);
    const n = manifold.mWorldSpaceNormal;
    const normal = { x: n.GetX(), y: n.GetY(), z: n.GetZ() };
    const p = manifold.GetWorldSpaceContactPointOn1(0);
    const point = { x: p.GetX(), y: p.GetY(), z: p.GetZ() };
    // Jolt adds this step's gravity before finding contacts; remove it so a body resting on (or
    // rolling onto) a surface does not look like a 0.08 m/s impact.
    const v1 = pointVelocity(r1.body, point, r1.motion === 'dynamic' ? this.gravityStep : 0);
    const v2 = pointVelocity(r2.body, point, r2.motion === 'dynamic' ? this.gravityStep : 0);
    // The normal points from body 1 to body 2: they approach when body 1 is faster along it.
    const speed = (v1.x - v2.x) * normal.x + (v1.y - v2.y) * normal.y + (v1.z - v2.z) * normal.z;
    if (speed < IMPACT_MIN_SPEED) return;

    const [a, b] = r1.handle < r2.handle ? [r1, r2] : [r2, r1];
    const key = `${a.handle}|${b.handle}`;
    if ((this.stepImpacts.get(key)?.speed ?? 0) >= speed) return;
    const mass =
      Number.isFinite(a.mass) && Number.isFinite(b.mass)
        ? (a.mass * b.mass) / (a.mass + b.mass)
        : Math.min(a.mass, b.mass);
    this.stepImpacts.set(key, {
      bodies: [a.handle, b.handle],
      speed,
      energy: 0.5 * mass * speed * speed,
    });
  }

  private guard(work: () => void): void {
    try {
      work();
    } catch (error) {
      this.callbackError ??= error;
    }
  }
}

function createdShape(settings: Jolt.ShapeSettings): Jolt.Shape {
  const result = settings.Create();
  if (result.HasError()) throw new Error(`Invalid physics shape: ${result.GetError().c_str()}`);
  return result.Get();
}

function convexRadius(...halfSides: number[]): number {
  return Math.min(CONVEX_RADIUS, 0.5 * Math.min(...halfSides));
}

/**
 * Velocity of a body's material point at `point` (world): v + ω × (p − centre of mass), with the
 * vertical speed `gravityStep` (g·dt) gained in the current step added back.
 */
function pointVelocity(body: Jolt.Body, point: Vec3, gravityStep: number): Vec3 {
  const lv = body.GetLinearVelocity();
  const v = { x: lv.GetX(), y: lv.GetY() + gravityStep, z: lv.GetZ() };
  const av = body.GetAngularVelocity();
  const w = { x: av.GetX(), y: av.GetY(), z: av.GetZ() };
  const c = body.GetCenterOfMassPosition();
  const r = { x: point.x - c.GetX(), y: point.y - c.GetY(), z: point.z - c.GetZ() };
  return {
    x: v.x + w.y * r.z - w.z * r.y,
    y: v.y + w.z * r.x - w.x * r.z,
    z: v.z + w.x * r.y - w.y * r.x,
  };
}

/** 32-bit FNV-1a hash as 8 hex digits. */
function fnv1a(bytes: Uint8Array): string {
  let hash = 0x811c9dc5;
  for (const byte of bytes) hash = Math.imul(hash ^ byte, 0x01000193) >>> 0;
  return hash.toString(16).padStart(8, '0');
}
