import { chainLayout } from '@intracraft/geometry';
import { COLORS, Box, CHAIN_HEIGHT, ChainStrands, FRAME_HEIGHT, Legs, SideFrames, n, safe } from './parts';
import type { ModelProps } from './types';

const BEAM_HEIGHT = 0.06;

export function ChainConveyorModel({ params }: ModelProps) {
  const length = n(params, 'length');
  const width = n(params, 'usableWidth');
  const height = n(params, 'surfaceHeight');
  const strands = safe(() => chainLayout(width, n(params, 'chainCount', 3)), []);
  const supportWidth = n(params, 'chainSupportWidth', 0.04);
  return (
    <>
      <ChainStrands x0={0} length={length} zPositions={strands} supportWidth={supportWidth} top={height} />
      {/* Guide beam under each strand. */}
      {strands.map((z) => (
        <Box
          key={z}
          size={[length, BEAM_HEIGHT, supportWidth + 0.02]}
          position={[length / 2, height - CHAIN_HEIGHT - BEAM_HEIGHT / 2, z]}
          color={COLORS.frame}
        />
      ))}
      <SideFrames x0={0} length={length} width={width} top={height - CHAIN_HEIGHT} />
      <Legs x0={0} length={length} width={width} height={height - CHAIN_HEIGHT - FRAME_HEIGHT} />
    </>
  );
}
