// Photocell drawn in its host conveyor's frame (the instance group sits on the host): sensor and
// reflector on thin posts and the beam between them, green while free and red while detecting.
import { instanceParams, num } from '@intracraft/library';
import { useFrame } from '@react-three/fiber';
import { useMemo, useRef } from 'react';
import { type MeshBasicMaterial, Quaternion, Vector3 } from 'three';
import { registry, useLayoutStore } from '../../layout-store';
import { liveFrame } from '../../sim-store';
import { Box, Mat, n } from './parts';
import type { ModelProps } from './types';

const BEAM_RADIUS = 0.008;
const POST = 0.02;
const FREE = '#3ddc84';
const DETECTING = '#ff4d4d';
const UP = new Vector3(0, 1, 0);
/** Surface height used while the sensor is not mounted. */
const UNMOUNTED_HEIGHT = 0.6;

export function PhotoeyeModel({ params, instanceId }: ModelProps) {
  const hostId = typeof params['host'] === 'string' ? params['host'] : '';
  const host = useLayoutStore((s) => s.layout.instances.find((i) => i.id === hostId));
  const surface = host === undefined ? UNMOUNTED_HEIGHT : num(instanceParams(registry.get(host.definitionId), host.params), 'surfaceHeight', 0);
  const diffuse = params['type'] === 'diffuse';
  const [sx, sy, sz] = [n(params, 'sensorS'), surface + n(params, 'sensorHeight'), n(params, 'sensorLateral')];
  const [rx, ry, rz] = diffuse
    ? [sx, sy, -sz]
    : [n(params, 'reflectorS'), surface + n(params, 'reflectorHeight'), n(params, 'reflectorLateral')];
  const { sensor, reflector, beam } = useMemo(() => {
    const from = new Vector3(sx, sy, sz);
    const to = new Vector3(rx, ry, rz);
    const direction = to.clone().sub(from);
    return {
      sensor: from,
      reflector: to,
      beam: {
        length: direction.length(),
        middle: from.clone().add(to).multiplyScalar(0.5),
        rotation: new Quaternion().setFromUnitVectors(UP, direction.clone().normalize()),
      },
    };
  }, [sx, sy, sz, rx, ry, rz]);
  const beamMaterial = useRef<MeshBasicMaterial>(null);

  useFrame(() => {
    if (beamMaterial.current === null || instanceId === undefined) return;
    beamMaterial.current.color.set(liveFrame.snapshot?.status[instanceId] === 'detecting' ? DETECTING : FREE);
  });

  return (
    <>
      <Post at={sensor} />
      <Box size={[0.05, 0.08, 0.05]} position={[sensor.x, sensor.y, sensor.z]} color="#e0b341" />
      {!diffuse && (
        <>
          <Post at={reflector} />
          <Box size={[0.02, 0.08, 0.07]} position={[reflector.x, reflector.y, reflector.z]} color="#c2453d" />
        </>
      )}
      {params['showBeam'] !== false && beam.length > 0 && (
        <mesh position={beam.middle} quaternion={beam.rotation} userData={{ noPick: true }}>
          <cylinderGeometry args={[BEAM_RADIUS, BEAM_RADIUS, beam.length, 6]} />
          <meshBasicMaterial ref={beamMaterial} color={FREE} transparent opacity={0.85} />
        </mesh>
      )}
    </>
  );
}

function Post({ at }: { at: Vector3 }) {
  return (
    <mesh position={[at.x, at.y / 2, at.z]} castShadow>
      <boxGeometry args={[POST, at.y, POST]} />
      <Mat color="#5b6470" />
    </mesh>
  );
}
