import { FRAME_HEIGHT, Legs, RollerBed, SideFrames, n } from './parts';
import type { ModelProps } from './types';

export function RollerConveyorModel({ params }: ModelProps) {
  const length = n(params, 'length');
  const width = n(params, 'usableWidth');
  const height = n(params, 'surfaceHeight');
  return (
    <>
      <RollerBed
        x0={0}
        length={length}
        width={width}
        surfaceHeight={height}
        pitch={n(params, 'rollerPitch')}
        diameter={n(params, 'rollerDiameter')}
      />
      <SideFrames x0={0} length={length} width={width} top={height} />
      <Legs x0={0} length={length} width={width} height={height - FRAME_HEIGHT} />
    </>
  );
}
