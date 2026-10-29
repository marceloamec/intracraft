// Shared building blocks of the parametric conveyor models (SI units, local frame: x along
// travel, y up, z across). Our own simple geometry; no third-party models (AGENTS.md, clean-room).
import { rollerLayout } from '@intracraft/geometry';
import { createContext, useContext, useLayoutEffect, useMemo, useRef } from 'react';
import { type InstancedMesh, Object3D } from 'three';

export const FRAME_HEIGHT = 0.12;
export const FRAME_THICKNESS = 0.05;
export const CHAIN_HEIGHT = 0.03;
export const COLORS = {
  frame: '#2f6fb3',
  roller: '#9aa3ad',
  chain: '#3b3f45',
  leg: '#5b6470',
  base: '#4a525d',
  deck: '#6d7782',
  sweep: '#e0b341',
};

/** How a model is drawn: normal, as a drag preview (ghost), selected, or dimmed while being moved. */
export interface ModelStyle {
  ghost: boolean;
  selected: boolean;
  dimmed: boolean;
}

export const ModelStyleContext = createContext<ModelStyle>({ ghost: false, selected: false, dimmed: false });

export function Mat({ color, metalness = 0.2, roughness = 0.6 }: { color: string; metalness?: number; roughness?: number }) {
  const { ghost, selected, dimmed } = useContext(ModelStyleContext);
  const translucent = ghost || dimmed;
  return (
    <meshStandardMaterial
      color={color}
      metalness={metalness}
      roughness={roughness}
      transparent={translucent}
      opacity={ghost ? 0.5 : dimmed ? 0.3 : 1}
      depthWrite={!translucent}
      emissive={selected ? '#2a6fd6' : '#000000'}
      emissiveIntensity={selected ? 0.45 : 0}
    />
  );
}

export function Box({
  size,
  position,
  color,
}: {
  size: [number, number, number];
  position: [number, number, number];
  color: string;
}) {
  return (
    <mesh position={position} castShadow receiveShadow>
      <boxGeometry args={size} />
      <Mat color={color} />
    </mesh>
  );
}

/** Rollers across the conveyor, one InstancedMesh (one draw call), placed by `rollerLayout`. */
export function RollerBed({
  x0,
  length,
  width,
  surfaceHeight,
  pitch,
  diameter,
}: {
  x0: number;
  length: number;
  width: number;
  surfaceHeight: number;
  pitch: number;
  diameter: number;
}) {
  const ref = useRef<InstancedMesh>(null);
  const positions = useMemo(() => safe(() => rollerLayout(length, pitch, diameter).positions, []), [length, pitch, diameter]);

  useLayoutEffect(() => {
    const mesh = ref.current;
    if (mesh === null) return;
    const dummy = new Object3D();
    positions.forEach((x, i) => {
      dummy.position.set(x0 + x, surfaceHeight - diameter / 2, 0);
      dummy.rotation.set(Math.PI / 2, 0, 0); // cylinder axis across the conveyor (z)
      dummy.updateMatrix();
      mesh.setMatrixAt(i, dummy.matrix);
    });
    mesh.instanceMatrix.needsUpdate = true;
  }, [positions, x0, surfaceHeight, diameter]);

  if (positions.length === 0) return null;
  return (
    <instancedMesh key={positions.length} ref={ref} args={[undefined, undefined, positions.length]} castShadow receiveShadow>
      <cylinderGeometry args={[diameter / 2, diameter / 2, width, 20]} />
      <Mat color={COLORS.roller} metalness={0.8} roughness={0.35} />
    </instancedMesh>
  );
}

/** Chain strands running along x, their top on the support plane. */
export function ChainStrands({
  x0,
  length,
  zPositions,
  supportWidth,
  top,
}: {
  x0: number;
  length: number;
  zPositions: number[];
  supportWidth: number;
  top: number;
}) {
  return (
    <>
      {zPositions.map((z) => (
        <Box
          key={z}
          size={[length, CHAIN_HEIGHT, supportWidth]}
          position={[x0 + length / 2, top - CHAIN_HEIGHT / 2, z]}
          color={COLORS.chain}
        />
      ))}
    </>
  );
}

/** Two side frames along x whose top edge is at `top`. */
export function SideFrames({ x0, length, width, top }: { x0: number; length: number; width: number; top: number }) {
  return (
    <>
      {[-1, 1].map((side) => (
        <Box
          key={side}
          size={[length, FRAME_HEIGHT, FRAME_THICKNESS]}
          position={[x0 + length / 2, top - FRAME_HEIGHT / 2, side * (width / 2 + FRAME_THICKNESS / 2)]}
          color={COLORS.frame}
        />
      ))}
    </>
  );
}

/** Four legs under a straight section. */
export function Legs({ x0, length, width, height }: { x0: number; length: number; width: number; height: number }) {
  const z = width / 2 + FRAME_THICKNESS / 2;
  const xs = [x0 + 0.1, x0 + length - 0.1];
  return (
    <>
      {xs.flatMap((x) =>
        [-z, z].map((zz) => (
          <Box key={`${x}:${zz}`} size={[0.05, height, 0.05]} position={[x, height / 2, zz]} color={COLORS.leg} />
        )),
      )}
    </>
  );
}

/** Runs a geometry helper; out-of-range parameters fall back instead of breaking the whole view. */
export function safe<T>(compute: () => T, fallback: T): T {
  try {
    return compute();
  } catch {
    return fallback;
  }
}

export function n(params: Record<string, number | boolean | string>, key: string, fallback = 0): number {
  const value = params[key];
  return typeof value === 'number' ? value : fallback;
}
