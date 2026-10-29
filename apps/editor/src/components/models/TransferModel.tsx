// Roller section with liftable transfer chains between the rollers, drawn in the lowered position.
import { transferChainPositions } from '@intracraft/geometry';
import { COLORS, Box, CHAIN_HEIGHT, FRAME_HEIGHT, Legs, RollerBed, SideFrames, n, safe } from './parts';
import type { ModelProps } from './types';

export function TransferModel({ params }: ModelProps) {
  const length = n(params, 'length');
  const width = n(params, 'usableWidth');
  const height = n(params, 'surfaceHeight');
  const pitch = n(params, 'rollerPitch');
  const diameter = n(params, 'rollerDiameter');
  const chainWidth = n(params, 'transferChainWidth', 0.04);
  const chainTop = height - n(params, 'lowerClearance');
  const chains = safe(() => transferChainPositions(length, pitch, diameter, n(params, 'transferChainCount', 3)), []);

  return (
    <>
      <RollerBed x0={0} length={length} width={width} surfaceHeight={height} pitch={pitch} diameter={diameter} />
      {chains.map((x) => (
        <Box
          key={x}
          size={[chainWidth, CHAIN_HEIGHT, width]}
          position={[x, chainTop - CHAIN_HEIGHT / 2, 0]}
          color={COLORS.chain}
        />
      ))}
      <SideFrames x0={0} length={length} width={width} top={height} />
      <Legs x0={0} length={length} width={width} height={height - FRAME_HEIGHT} />
    </>
  );
}
