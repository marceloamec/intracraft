import type { Instance } from '@intracraft/core-model';
import { createBuiltInRegistry } from '@intracraft/library';
import { describe, expect, it } from 'vitest';
import { signalCatalog } from './signals';

const REGISTRY = createBuiltInRegistry();

describe('signal catalogue', () => {
  it('lists conveyor, source and sensor signals by instance id', () => {
    const catalog = signalCatalog(
      { instances: [instance('source1', 'flow.source'), instance('roller1', 'conveyor.roller'), instance('photoeye1', 'sensor.photoeye')], connections: [] },
      REGISTRY,
    );
    expect(catalog.writable).toEqual(['source1.enable', 'roller1.run']);
    expect(catalog.readable).toEqual(['roller1.running', 'roller1.occupied', 'photoeye1.detected', 'source1.enable', 'roller1.run']);
  });

  it('uses the custom output name of a sensor', () => {
    const catalog = signalCatalog({ instances: [instance('photoeye1', 'sensor.photoeye', { outputSignal: 'B12' })], connections: [] }, REGISTRY);
    expect(catalog.readable).toEqual(['B12']);
  });
});

function instance(id: string, definitionId: string, params: Record<string, string> = {}): Instance {
  return { id, definitionId, definitionVersion: '0.1.0', name: id, position: { x: 0, y: 0, z: 0 }, yaw: 0, params, attributes: {} };
}
