// Manual lift table and scissor lift table (S12, S13): a roller deck on a scissor lift standing on a
// base frame. Origin at the entry end (x = 0) on the centre line, on the floor. During a run the deck
// follows the height the simulation reports for this instance; otherwise it rests at its rest height.
import { useFrame } from '@react-three/fiber';
import { useRef } from 'react';
import type { Group, Mesh } from 'three';
import { liveFrame } from '../../sim-store';
import { COLORS, Box, FRAME_HEIGHT, Mat, RollerBed, SideFrames, n } from './parts';
import type { ModelProps } from './types';

const BASE_HEIGHT = 0.06;
const ARM_SECTION = 0.04;
/** Distance of the scissor pins from the deck ends (m). */
const PIN_INSET = 0.2;
const OPERATOR_COLOR = '#e0b341';

export function LiftTableModel({ params, instanceId, variant }: ModelProps & { variant: 'manual' | 'scissor' }) {
  const length = n(params, 'length');
  const width = n(params, 'usableWidth');
  const rest = restHeight(params, variant);
  const deck = useRef<Group>(null);
  const arms = useRef<(Mesh | null)[]>([]);

  useFrame(() => {
    const height = (instanceId === undefined ? undefined : liveFrame.snapshot?.heights[instanceId]) ?? rest;
    if (deck.current !== null) deck.current.position.y = height;
    placeArms(arms.current, length, height);
  });

  return (
    <>
      {/* Base frame on the floor. */}
      <Box size={[length, BASE_HEIGHT, width]} position={[length / 2, BASE_HEIGHT / 2, 0]} color={COLORS.base} />
      {/* Two crossed arms on each side; placed every frame from the deck height. */}
      {[0, 1, 2, 3].map((i) => (
        <mesh
          key={i}
          ref={(mesh) => {
            arms.current[i] = mesh;
          }}
          position={[length / 2, rest / 2, (i < 2 ? -1 : 1) * (width / 2 - ARM_SECTION)]}
          castShadow
        >
          <boxGeometry args={[1, ARM_SECTION, ARM_SECTION]} />
          <Mat color={COLORS.leg} metalness={0.5} />
        </mesh>
      ))}
      <group ref={deck} position={[0, rest, 0]}>
        <RollerBed
          x0={0}
          length={length}
          width={width - 0.05}
          surfaceHeight={0}
          pitch={n(params, 'rollerPitch')}
          diameter={n(params, 'rollerDiameter')}
        />
        <SideFrames x0={0} length={length} width={width} top={0} />
        <Box
          size={[length, 0.02, width]}
          position={[length / 2, -FRAME_HEIGHT + 0.01, 0]}
          color={COLORS.deck}
        />
      </group>
      {variant === 'manual' && (
        // Floor marking where the operator drives the pallet jack in (OP side).
        <mesh position={[-0.6, 0.003, 0]} rotation={[-Math.PI / 2, 0, 0]} userData={{ noPick: true }}>
          <planeGeometry args={[1.2, width]} />
          <meshBasicMaterial color={OPERATOR_COLOR} transparent opacity={0.35} />
        </mesh>
      )}
    </>
  );
}

/** Deck height while idle: the entry height (the operator side of an input table). */
function restHeight(params: ModelProps['params'], variant: 'manual' | 'scissor'): number {
  if (variant === 'scissor') return n(params, 'entryHeight');
  return params['flowDirection'] === 'out' ? n(params, 'conveyorHeight') : n(params, 'lowHeight');
}

/** Crossed arms from the base pins to the deck pins: each one spans the stroke diagonally. */
function placeArms(arms: (Mesh | null)[], length: number, height: number): void {
  const bottom = BASE_HEIGHT;
  const top = height - FRAME_HEIGHT;
  const x0 = PIN_INSET;
  const x1 = length - PIN_INSET;
  arms.forEach((arm, i) => {
    if (arm === null) return;
    const rising = i % 2 === 0;
    const dy = (top - bottom) * (rising ? 1 : -1);
    const dx = x1 - x0;
    arm.position.set((x0 + x1) / 2, (top + bottom) / 2, arm.position.z);
    arm.rotation.set(0, 0, Math.atan2(dy, dx));
    arm.scale.set(Math.max(Math.hypot(dx, dy), 0.01), 1, 1);
  });
}
