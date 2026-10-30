// Source and sink: a small stand 0.4 m long at port height, green with an outgoing arrow (source)
// or red with an incoming arrow (sink). They mark where pallets enter and leave the model.
import { Box, Mat, n } from './parts';
import type { ModelProps } from './types';

const DEPTH = 0.4;
const WIDTH = 0.9;
const PLATE = 0.04;
const POST = 0.06;
const ARROW_LENGTH = 0.22;
const ARROW_RADIUS = 0.1;

export function FlowEndModel({ params, kind }: ModelProps & { kind: 'source' | 'sink' }) {
  const height = n(params, 'surfaceHeight', 0.6);
  const color = kind === 'source' ? '#2f9e5b' : '#c2453d';
  // Arrow pointing along +x: out of the source, into the sink.
  const arrowX = kind === 'source' ? DEPTH - ARROW_LENGTH / 2 : ARROW_LENGTH / 2;
  return (
    <>
      <Box size={[DEPTH, PLATE, WIDTH]} position={[DEPTH / 2, height - PLATE / 2, 0]} color={color} />
      <Box size={[POST, height - PLATE, POST]} position={[DEPTH / 2, (height - PLATE) / 2, 0]} color="#5b6470" />
      <mesh position={[arrowX, height + ARROW_RADIUS, 0]} rotation={[0, 0, -Math.PI / 2]}>
        <coneGeometry args={[ARROW_RADIUS, ARROW_LENGTH, 16]} />
        <Mat color={color} />
      </mesh>
    </>
  );
}
