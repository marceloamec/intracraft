// Pallets of the running simulation, drawn from the latest worker frame. Each pallet is a wooden
// base plus a load block, as two InstancedMeshes (one draw call each); the load is red while its
// conveyor is blocked and amber while the controller has it switched off. Between frames the position
// is extrapolated with the pallet's speed and acceleration so the motion looks smooth.
import { instanceParams, num } from '@intracraft/library';
import { useFrame } from '@react-three/fiber';
import { useLayoutEffect, useMemo, useRef } from 'react';
import { Color, type InstancedMesh, Matrix4, Quaternion, Vector3 } from 'three';
import { registry, useLayoutStore } from '../layout-store';
import { liveFrame, useSimStore } from '../sim-store';

const CAPACITY = 1024;
/** Longest wall-clock time a frame is extrapolated (ms). */
const MAX_EXTRAPOLATION_MS = 50;
const LOAD_INSET = 0.96;
const COLOR_BASE = new Color('#b0865a');
const COLOR_LOAD = new Color('#d8b77a');
const COLOR_BLOCKED = new Color('#d0574a');
const COLOR_STOPPED = new Color('#e0b341');
const UP = new Vector3(0, 1, 0);

interface Carrier {
  x: number;
  z: number;
  yaw: number;
  height: number;
}

export function PalletLayer() {
  const layout = useLayoutStore((s) => s.layout);
  const bases = useRef<InstancedMesh>(null);
  const loads = useRef<InstancedMesh>(null);
  const carriers = useMemo(() => {
    const map = new Map<string, Carrier>();
    for (const instance of layout.instances) {
      const params = instanceParams(registry.get(instance.definitionId), instance.params);
      map.set(instance.id, { x: instance.position.x, z: instance.position.z, yaw: instance.yaw, height: num(params, 'surfaceHeight', 0) });
    }
    return map;
  }, [layout]);
  // Create the per-instance colours before the first draw so the material compiles with them.
  useLayoutEffect(() => {
    const load = loads.current;
    if (load === null) return;
    for (let i = 0; i < CAPACITY; i++) load.setColorAt(i, COLOR_LOAD);
  }, []);
  const scratch = useMemo(() => ({ matrix: new Matrix4(), rotation: new Quaternion(), position: new Vector3(), scale: new Vector3() }), []);

  useFrame(() => {
    const base = bases.current;
    const load = loads.current;
    if (base === null || load === null) return;
    const snapshot = liveFrame.snapshot;
    const pallets = snapshot?.pallets.slice(0, CAPACITY) ?? [];
    const speed = useSimStore.getState().speed;
    const factor = liveFrame.playing && (speed === 1 || speed === 10) ? speed : 0;
    const ahead = (Math.min(performance.now() - liveFrame.receivedAt, MAX_EXTRAPOLATION_MS) / 1000) * factor;

    const { matrix, rotation, position, scale } = scratch;
    pallets.forEach((pallet, i) => {
      const carrier = carriers.get(pallet.carrier);
      if (carrier === undefined) return;
      const dt = pallet.accel < 0 ? Math.min(ahead, pallet.speed / -pallet.accel) : ahead;
      const front = pallet.front + pallet.speed * dt + 0.5 * pallet.accel * dt * dt;
      // Pallet centre on the carrier's path (its own frame), then into the world by the carrier pose.
      const along = front - pallet.length / 2;
      const localX = pallet.pathX + along * Math.cos(pallet.heading);
      const localZ = pallet.pathZ - along * Math.sin(pallet.heading);
      const cos = Math.cos(carrier.yaw);
      const sin = Math.sin(carrier.yaw);
      const x = carrier.x + localX * cos + localZ * sin;
      const z = carrier.z - localX * sin + localZ * cos;
      rotation.setFromAxisAngle(UP, carrier.yaw + pallet.heading);

      const floor = carrier.height + pallet.pathY;
      position.set(x, floor + pallet.height / 2, z);
      scale.set(pallet.length, pallet.height, pallet.width);
      base.setMatrixAt(i, matrix.compose(position, rotation, scale));

      const loadHeight = Math.max(pallet.loadHeight, 0.01);
      position.set(x, floor + pallet.height + loadHeight / 2, z);
      scale.set(pallet.length * LOAD_INSET, loadHeight, pallet.width * LOAD_INSET);
      load.setMatrixAt(i, matrix.compose(position, rotation, scale));
      const status = snapshot?.status[pallet.carrier];
      load.setColorAt(i, status === 'blocked' ? COLOR_BLOCKED : status === 'stopped' ? COLOR_STOPPED : COLOR_LOAD);
    });
    base.count = pallets.length;
    load.count = pallets.length;
    base.instanceMatrix.needsUpdate = true;
    load.instanceMatrix.needsUpdate = true;
    if (load.instanceColor !== null) load.instanceColor.needsUpdate = true;
  });

  return (
    <group userData={{ noPick: true }}>
      <instancedMesh ref={bases} args={[undefined, undefined, CAPACITY]} count={0} castShadow frustumCulled={false}>
        <boxGeometry args={[1, 1, 1]} />
        <meshStandardMaterial color={COLOR_BASE} roughness={0.9} />
      </instancedMesh>
      <instancedMesh ref={loads} args={[undefined, undefined, CAPACITY]} count={0} castShadow frustumCulled={false}>
        <boxGeometry args={[1, 1, 1]} />
        <meshStandardMaterial roughness={0.8} />
      </instancedMesh>
    </group>
  );
}
