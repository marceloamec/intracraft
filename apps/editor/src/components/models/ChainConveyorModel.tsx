// Chain conveyor (S02): chain strands on guide-rail beams, one frame section under each strand, a
// drive shaft with a sprocket per strand (centre drive: the chains loop down to it mid-length; head
// drive: at the end), the geared motor on the drive side, optional side guards and end stop.
import { chainLayout } from '@intracraft/geometry';
import {
  COLORS,
  Box,
  CHAIN_HEIGHT,
  ChainStrands,
  DriveUnit,
  EndStopPlate,
  FRAME_HEIGHT,
  FRAME_THICKNESS,
  Legs,
  Mat,
  SideFrames,
  SideGuard,
  n,
  safe,
} from './parts';
import type { ModelProps } from './types';

const BEAM_HEIGHT = 0.06;
const SPROCKET_RADIUS = 0.06;
/** The drive shaft lies this far below the chain top (m). */
const SHAFT_DROP = 0.22;
/** A centre drive pulls each strand down over this length on both sides of the shaft (m). */
const LOOP_HALF = 0.18;

export function ChainConveyorModel({ params }: ModelProps) {
  const length = n(params, 'length');
  const width = n(params, 'usableWidth');
  const height = n(params, 'surfaceHeight');
  const strands = safe(() => chainLayout(width, n(params, 'chainCount', 3)), []);
  const supportWidth = n(params, 'chainSupportWidth', 0.04);
  const side = params['driveSide'] === 'left' ? -1 : 1;
  const head = params['drivePosition'] === 'end';
  const driveX = head ? length - SPROCKET_RADIUS : length / 2;
  const shaftY = height - (head ? CHAIN_HEIGHT / 2 + SPROCKET_RADIUS : SHAFT_DROP);
  const frameTop = height - CHAIN_HEIGHT;
  const guards = { none: [], right: [1], left: [-1], both: [-1, 1] }[String(params['sideGuard'] ?? 'none')] ?? [];
  const guardZ = width / 2 + FRAME_THICKNESS + 0.03;
  return (
    <>
      <ChainStrands x0={0} length={length} zPositions={strands} supportWidth={supportWidth} top={height} />
      {strands.map((z) => (
        <Box
          key={z}
          size={[length, BEAM_HEIGHT, supportWidth + 0.02]}
          position={[length / 2, frameTop - BEAM_HEIGHT / 2, z]}
          color={COLORS.frame}
        />
      ))}
      {/* Drive shaft across all strands, a sprocket under each strand and, for a centre drive, the chain loop. */}
      <mesh position={[driveX, shaftY, side * 0.05]} rotation={[Math.PI / 2, 0, 0]} castShadow>
        <cylinderGeometry args={[0.025, 0.025, width + 0.1, 12]} />
        <Mat color={COLORS.roller} metalness={0.8} />
      </mesh>
      {strands.map((z) => (
        <mesh key={z} position={[driveX, shaftY, z]} rotation={[Math.PI / 2, 0, 0]} castShadow>
          <cylinderGeometry args={[SPROCKET_RADIUS, SPROCKET_RADIUS, supportWidth, 16]} />
          <Mat color={COLORS.chain} metalness={0.6} />
        </mesh>
      ))}
      {!head &&
        strands.flatMap((z) =>
          [-1, 1].map((dir) => {
            const dx = LOOP_HALF - SPROCKET_RADIUS;
            const dy = frameTop - BEAM_HEIGHT - shaftY;
            return (
              <mesh
                key={`${z}:${dir}`}
                position={[driveX + (dir * (LOOP_HALF + SPROCKET_RADIUS)) / 2, shaftY + dy / 2, z]}
                rotation={[0, 0, -dir * Math.atan2(dx, dy)]}
              >
                <boxGeometry args={[0.012, Math.hypot(dx, dy), supportWidth * 0.8]} />
                <Mat color={COLORS.chain} />
              </mesh>
            );
          }),
        )}
      <DriveUnit position={[driveX, shaftY, side * (width / 2 + FRAME_THICKNESS + 0.12)]} axis="z" side={side} />
      {guards.map((s) => (
        <SideGuard key={s} x0={0} length={length} z={s * guardZ} top={height} />
      ))}
      {params['endStop'] === true && <EndStopPlate x={length} width={width} top={height} />}
      <SideFrames x0={0} length={length} width={width} top={frameTop} />
      <Legs x0={0} length={length} width={width} height={frameTop - FRAME_HEIGHT} />
    </>
  );
}
