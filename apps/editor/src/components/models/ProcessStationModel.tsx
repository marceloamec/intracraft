// Process station: a short roller or chain deck under a gantry that stands for the work being done.
// Origin at the entry (port IN) on the centre line; the exit (port OUT) is at x = length.
import { COLORS, Box, ChainStrands, Legs, RollerBed, SideFrames, n } from './parts';
import type { ModelProps } from './types';

const DECK_WIDTH = 0.9;
const ROLLER_PITCH = 0.2063;
const ROLLER_DIAMETER = 0.089;
const CHAIN_Z = [-0.3, 0, 0.3];
const CHAIN_SUPPORT_WIDTH = 0.04;
const GANTRY_COLOR = '#c9822b';
const GANTRY_POST = 0.1;
const GANTRY_CLEARANCE = 1.9; // free height above the deck for a loaded pallet
const GANTRY_DEPTH = 0.3;

export function ProcessStationModel({ params }: ModelProps) {
  const length = n(params, 'length');
  const height = n(params, 'surfaceHeight');
  const top = height + GANTRY_CLEARANCE;
  const postZ = DECK_WIDTH / 2 + GANTRY_POST / 2;

  return (
    <>
      {params['deckType'] === 'chains' ? (
        <ChainStrands x0={0} length={length} zPositions={CHAIN_Z} supportWidth={CHAIN_SUPPORT_WIDTH} top={height} />
      ) : (
        <RollerBed
          x0={0}
          length={length}
          width={DECK_WIDTH - 0.1}
          surfaceHeight={height}
          pitch={ROLLER_PITCH}
          diameter={ROLLER_DIAMETER}
        />
      )}
      <SideFrames x0={0} length={length} width={DECK_WIDTH} top={height} />
      <Legs x0={0} length={length} width={DECK_WIDTH} height={height - 0.12} />
      {[-postZ, postZ].map((z) => (
        <Box
          key={z}
          size={[GANTRY_DEPTH, GANTRY_CLEARANCE, GANTRY_POST]}
          position={[length / 2, height + GANTRY_CLEARANCE / 2, z]}
          color={GANTRY_COLOR}
        />
      ))}
      <Box
        size={[GANTRY_DEPTH, GANTRY_POST, DECK_WIDTH + 2 * GANTRY_POST]}
        position={[length / 2, top + GANTRY_POST / 2, 0]}
        color={COLORS.frame}
      />
    </>
  );
}
