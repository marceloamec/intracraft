// Gravity roller line (S15): idle rollers on a slope with a brake roller every N rollers, a separator
// at the start of the pick-up position and an end stop. Origin at the entry (port IN) on the centre
// line, on the floor; the roller tops fall from the entry height to the exit height.
import { rollerLayout } from '@intracraft/geometry';
import { COLORS, Box, FRAME_HEIGHT, Mat, RollerBed, SideFrames, n, safe } from './parts';
import type { ModelProps } from './types';

const BRAKE_COLOR = '#c4473a';
const SEPARATOR_COLOR = '#e0b341';
/** Brake drum: a short sleeve around the middle of a roller (m). */
const BRAKE_LENGTH = 0.14;
const BRAKE_OVERSIZE = 0.006;
const STOP_HEIGHT = 0.15;
const SEPARATOR_HEIGHT = 0.08;
const PLATE = 0.02;
const LEG = 0.05;

export function GravityLineModel({ params }: ModelProps) {
  const length = n(params, 'length');
  const width = n(params, 'usableWidth');
  const slope = n(params, 'slope');
  const exit = n(params, 'exitHeight');
  const entry = exit + slope * length;
  const pitch = n(params, 'rollerPitch');
  const diameter = n(params, 'rollerDiameter');
  const brakeEvery = Math.max(1, Math.round(n(params, 'brakeEvery', 4)));
  // Distances along the sloped deck are longer than in plan by √(1 + slope²).
  const stretch = Math.sqrt(1 + slope * slope);
  const deck = length * stretch;
  const separator = (length - n(params, 'pickupLength')) * stretch;
  const brakes = safe(() => rollerLayout(deck, pitch, diameter).positions, []).filter((_, i) => i % brakeEvery === 0);

  return (
    <>
      <group position={[0, entry, 0]} rotation={[0, 0, -Math.atan(slope)]}>
        <RollerBed x0={0} length={deck} width={width - 0.05} surfaceHeight={0} pitch={pitch} diameter={diameter} />
        {brakes.map((x) => (
          <mesh key={x} position={[x, -diameter / 2, 0]} rotation={[Math.PI / 2, 0, 0]}>
            <cylinderGeometry args={[diameter / 2 + BRAKE_OVERSIZE, diameter / 2 + BRAKE_OVERSIZE, BRAKE_LENGTH, 20]} />
            <Mat color={BRAKE_COLOR} />
          </mesh>
        ))}
        <SideFrames x0={0} length={deck} width={width} top={0} />
        {/* Separator blade between two rollers, and the end stop at the pick-up end. */}
        <Box size={[PLATE, SEPARATOR_HEIGHT, width * 0.6]} position={[separator, SEPARATOR_HEIGHT / 2, 0]} color={SEPARATOR_COLOR} />
        <Box size={[PLATE, STOP_HEIGHT, width]} position={[deck + PLATE / 2, STOP_HEIGHT / 2, 0]} color={COLORS.frame} />
      </group>
      {/* Legs at both ends: longer at the high entry end. */}
      {[0.1, length - 0.1].flatMap((x) => {
        const height = entry - slope * x - FRAME_HEIGHT;
        return [-1, 1].map((side) => (
          <Box
            key={`${x}:${side}`}
            size={[LEG, height, LEG]}
            position={[x, height / 2, side * (width / 2 + LEG / 2)]}
            color={COLORS.leg}
          />
        ));
      })}
    </>
  );
}
