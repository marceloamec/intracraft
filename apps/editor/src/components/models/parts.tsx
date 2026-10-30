// Shared building blocks of the parametric conveyor models (SI units, local frame: x along
// travel, y up, z across). Our own simple geometry; no third-party models (AGENTS.md, clean-room).
import { frameBracing, rollerLayout, supportPositions } from '@intracraft/geometry';
import { Fragment, createContext, useContext, useLayoutEffect, useMemo, useRef } from 'react';
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
  /** Guards, end stops and guide disks. */
  guard: '#d9a62e',
  motor: '#3f5f7a',
};
const POST = 0.05;
/** Height of the adjustable foot under each post, plate included (m). */
const FOOT_HEIGHT = 0.06;
const FOOT_PLATE = 0.12;
const PLATE_THICKNESS = 0.01;
const BRACE = 0.035;

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

/**
 * Supports under a straight section whose frame bottom is at `height`: pairs of posts at most 2 m
 * apart, each on an adjustable foot with a floor plate; a cross beam from 650 mm conveying height
 * and a diagonal brace from 850 mm (indicative, from heavy pallet conveyors).
 */
export function Legs({ x0, length, width, height }: { x0: number; length: number; width: number; height: number }) {
  const z = width / 2 + FRAME_THICKNESS / 2;
  const post = height - FOOT_HEIGHT;
  const { crossBeam, diagonal } = frameBracing(height + FRAME_HEIGHT);
  const rise = post * 0.7;
  const brace = Math.hypot(2 * z, rise);
  return (
    <>
      {supportPositions(length).map((x) => (
        <Fragment key={x}>
          {[-z, z].map((zz) => (
            <Fragment key={zz}>
              <Box size={[POST, post, POST]} position={[x0 + x, FOOT_HEIGHT + post / 2, zz]} color={COLORS.leg} />
              <Box size={[FOOT_PLATE, PLATE_THICKNESS, FOOT_PLATE]} position={[x0 + x, PLATE_THICKNESS / 2, zz]} color={COLORS.base} />
              <mesh position={[x0 + x, (FOOT_HEIGHT + PLATE_THICKNESS) / 2, zz]} castShadow>
                <cylinderGeometry args={[0.012, 0.012, FOOT_HEIGHT - PLATE_THICKNESS, 8]} />
                <Mat color={COLORS.roller} metalness={0.7} />
              </mesh>
            </Fragment>
          ))}
          {crossBeam && <Box size={[BRACE, BRACE, 2 * z]} position={[x0 + x, FOOT_HEIGHT + post * 0.25, 0]} color={COLORS.leg} />}
          {diagonal && (
            <mesh position={[x0 + x, FOOT_HEIGHT + post * 0.25 + rise / 2, 0]} rotation={[-Math.atan2(rise, 2 * z), 0, 0]} castShadow>
              <boxGeometry args={[BRACE, BRACE, brace]} />
              <Mat color={COLORS.leg} />
            </mesh>
          )}
        </Fragment>
      ))}
    </>
  );
}

/** Cover over the roller drive chains on top of the frame on the drive side (`side` +1: right, −1: left). */
export function ChainGuard({ x0, length, width, top, side }: { x0: number; length: number; width: number; top: number; side: number }) {
  const depth = FRAME_THICKNESS + 0.04;
  return <Box size={[length, 0.03, depth]} position={[x0 + length / 2, top + 0.015, side * (width / 2 + depth / 2)]} color={COLORS.guard} />;
}

/** Geared motor: a gearbox with the motor on one side along `axis`. */
export function DriveUnit({ position, axis, side }: { position: [number, number, number]; axis: 'x' | 'z'; side: number }) {
  const [x, y, z] = position;
  const motor: [number, number, number] = axis === 'x' ? [x - 0.24, y, z] : [x, y, z + side * 0.24];
  return (
    <>
      <Box size={[0.22, 0.22, 0.2]} position={position} color={COLORS.motor} />
      <mesh position={motor} rotation={axis === 'x' ? [0, 0, Math.PI / 2] : [Math.PI / 2, 0, 0]} castShadow>
        <cylinderGeometry args={[0.085, 0.085, 0.28, 20]} />
        <Mat color={COLORS.motor} metalness={0.4} />
      </mesh>
    </>
  );
}

/** Flanged disks on some rollers that guide the pallet runners. */
export function GuideDisks({ xs, zs, top, diameter }: { xs: number[]; zs: number[]; top: number; diameter: number }) {
  return (
    <>
      {xs.flatMap((x) =>
        zs.map((z) => (
          <mesh key={`${x}:${z}`} position={[x, top - diameter / 2, z]} rotation={[Math.PI / 2, 0, 0]} castShadow>
            <cylinderGeometry args={[diameter / 2 + 0.02, diameter / 2 + 0.02, 0.012, 24]} />
            <Mat color={COLORS.guard} metalness={0.5} />
          </mesh>
        )),
      )}
    </>
  );
}

/** Fixed plate at the free end of a line, standing above the conveying surface. */
export function EndStopPlate({ x, width, top }: { x: number; width: number; top: number }) {
  return <Box size={[0.02, 0.15, width]} position={[x + 0.01, top + 0.045, 0]} color={COLORS.guard} />;
}

/** Rail on posts along one side, at the height of the pallet side. */
export function SideGuard({ x0, length, z, top }: { x0: number; length: number; z: number; top: number }) {
  const posts = supportPositions(length, 0.05, 1);
  return (
    <>
      {posts.map((x) => (
        <Box key={x} size={[0.04, 0.24, 0.04]} position={[x0 + x, top + 0.07, z]} color={COLORS.guard} />
      ))}
      <Box size={[length, 0.04, 0.04]} position={[x0 + length / 2, top + 0.17, z]} color={COLORS.guard} />
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
