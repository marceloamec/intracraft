// Pallet elevator (S14): two masts beside the deck, a cross beam on top and a carriage with a roller or
// chain deck that runs between the entry and the exit height. Origin at the entry end (x = 0) on the
// centre line, on the floor. During a run the carriage follows the height the simulation reports.
import { chainLayout } from '@intracraft/geometry';
import { useFrame } from '@react-three/fiber';
import { useRef } from 'react';
import type { Group } from 'three';
import { liveFrame } from '../../sim-store';
import { COLORS, Box, ChainStrands, FRAME_HEIGHT, RollerBed, SideFrames, n, safe } from './parts';
import type { ModelProps } from './types';

const MAST_SECTION = 0.16;
/** Gap between the deck frame and a mast (m). */
const MAST_GAP = 0.08;
/** Free height above the upper level for a loaded pallet (m). */
const HEADROOM = 2;
const CHAIN_SUPPORT_WIDTH = 0.04;
const MAST_COLOR = '#d9a62e';

export function ElevatorModel({ params, instanceId }: ModelProps) {
  const length = n(params, 'length');
  const width = n(params, 'usableWidth');
  const entry = n(params, 'entryHeight');
  const top = Math.max(entry, n(params, 'exitHeight')) + HEADROOM;
  const mastZ = width / 2 + MAST_GAP + MAST_SECTION / 2;
  const carriage = useRef<Group>(null);

  useFrame(() => {
    if (carriage.current === null) return;
    carriage.current.position.y = (instanceId === undefined ? undefined : liveFrame.snapshot?.heights[instanceId]) ?? entry;
  });

  return (
    <>
      {[-mastZ, mastZ].map((z) => (
        <Box key={z} size={[MAST_SECTION, top, MAST_SECTION]} position={[length / 2, top / 2, z]} color={MAST_COLOR} />
      ))}
      <Box
        size={[MAST_SECTION, MAST_SECTION, 2 * mastZ + MAST_SECTION]}
        position={[length / 2, top + MAST_SECTION / 2, 0]}
        color={MAST_COLOR}
      />
      <group ref={carriage} position={[0, entry, 0]}>
        {params['deckType'] === 'chains' ? (
          <ChainStrands
            x0={0}
            length={length}
            zPositions={safe(() => chainLayout(width, 3), [])}
            supportWidth={CHAIN_SUPPORT_WIDTH}
            top={0}
          />
        ) : (
          <RollerBed
            x0={0}
            length={length}
            width={width - 0.05}
            surfaceHeight={0}
            pitch={n(params, 'rollerPitch')}
            diameter={n(params, 'rollerDiameter')}
          />
        )}
        <SideFrames x0={0} length={length} width={width} top={0} />
        {/* Carriage arms from the deck frame to the masts. */}
        {[-1, 1].map((side) => (
          <Box
            key={side}
            size={[MAST_SECTION * 1.6, FRAME_HEIGHT, MAST_GAP]}
            position={[length / 2, -FRAME_HEIGHT / 2, side * (width / 2 + MAST_GAP / 2)]}
            color={COLORS.frame}
          />
        ))}
      </group>
    </>
  );
}
