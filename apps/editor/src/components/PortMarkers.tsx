// Small arrows on every port: green when connected, orange when free, cyan for the port the
// drag preview is about to snap onto.
import type { Layout } from '@intracraft/core-model';
import { type SnapLink, worldPorts } from '@intracraft/library';
import { useMemo } from 'react';
import { registry } from '../layout-store';

const MARKER_LIFT = 0.06; // m above the support plane
const COLORS = { connected: '#3fb950', free: '#d29922', target: '#39c5e6' };

export function PortMarkers({ layout, target }: { layout: Layout; target: SnapLink['target'] | null }) {
  const ports = useMemo(() => {
    const connected = new Set(
      layout.connections.flatMap((c) => [`${c.from.instanceId}.${c.from.portId}`, `${c.to.instanceId}.${c.to.portId}`]),
    );
    return layout.instances.flatMap((instance) =>
      worldPorts(instance, registry.get(instance.definitionId)).map((port) => {
        const key = `${port.instanceId}.${port.portId}`;
        return { key, port, connected: connected.has(key) };
      }),
    );
  }, [layout]);

  return (
    <>
      {ports.map(({ key, port, connected }) => {
        const isTarget = target !== null && key === `${target.instanceId}.${target.portId}`;
        const color = isTarget ? COLORS.target : connected ? COLORS.connected : COLORS.free;
        return (
          <group
            key={key}
            position={[port.position.x, port.position.y + MARKER_LIFT, port.position.z]}
            rotation={[0, Math.atan2(-port.facing.z, port.facing.x), 0]}
          >
            <mesh rotation={[0, 0, -Math.PI / 2]} scale={isTarget ? 1.6 : 1}>
              <coneGeometry args={[0.04, 0.1, 12]} />
              <meshBasicMaterial color={color} />
            </mesh>
          </group>
        );
      })}
    </>
  );
}
