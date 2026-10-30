// Chain and roller turntables share the fixed base, the platform and the swept circle; only the
// deck differs. Origin at the platform centre. During a run the platform follows the angle the
// simulation reports for this instance.
import { chainLayout } from '@intracraft/geometry';
import { useFrame } from '@react-three/fiber';
import { useRef } from 'react';
import { DoubleSide, type Group } from 'three';
import { liveFrame } from '../../sim-store';
import { COLORS, Box, CHAIN_HEIGHT, ChainStrands, Mat, RollerBed, n, safe } from './parts';
import type { ModelProps } from './types';

const DECK_THICKNESS = 0.04;
const DECK_GAP = 0.05; // between platform plate and support plane
const BASE_RADIUS = 0.35;

export function TurntableModel({
  params,
  deck,
  instanceId,
}: ModelProps & { deck: 'chains' | 'rollers' }) {
  const platform = useRef<Group>(null);
  useFrame(() => {
    if (platform.current === null || instanceId === undefined) return;
    platform.current.rotation.y = liveFrame.snapshot?.angles[instanceId] ?? 0;
  });
  const length = n(params, 'platformLength');
  const width = n(params, 'platformWidth');
  const height = n(params, 'surfaceHeight');
  const plateTop =
    height - DECK_GAP - (deck === 'chains' ? CHAIN_HEIGHT : n(params, 'rollerDiameter') / 2);
  const baseHeight = plateTop - DECK_THICKNESS;
  const swept = 0.5 * Math.hypot(length, width) + n(params, 'sweepClearance');

  return (
    <>
      <mesh position={[0, baseHeight / 2, 0]} castShadow receiveShadow>
        <cylinderGeometry args={[BASE_RADIUS, BASE_RADIUS * 1.15, baseHeight, 32]} />
        <Mat color={COLORS.base} />
      </mesh>
      <group ref={platform}>
        <Box
          size={[length, DECK_THICKNESS, width]}
          position={[0, plateTop - DECK_THICKNESS / 2, 0]}
          color={COLORS.deck}
        />
        {deck === 'rollers' ? (
          <RollerBed
            x0={-length / 2}
            length={length}
            width={width - 0.1}
            surfaceHeight={height}
            pitch={n(params, 'rollerPitch')}
            diameter={n(params, 'rollerDiameter')}
          />
        ) : (
          <ChainStrands
            x0={-length / 2}
            length={length}
            zPositions={safe(() => chainLayout(width, n(params, 'chainCount', 3)), [])}
            supportWidth={n(params, 'chainSupportWidth', 0.04)}
            top={height}
          />
        )}
      </group>
      {/* Swept circle on the floor: neighbours must stay outside it. */}
      <mesh position={[0, 0.003, 0]} rotation={[-Math.PI / 2, 0, 0]} userData={{ noPick: true }}>
        <ringGeometry args={[swept - 0.015, swept, 72]} />
        <meshBasicMaterial color={COLORS.sweep} transparent opacity={0.6} side={DoubleSide} />
      </mesh>
    </>
  );
}
