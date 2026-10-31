import { readProjectFile } from '@intracraft/core-model';
import { createBuiltInRegistry, deriveConnections } from '@intracraft/library';
import { buildFlowNetwork } from '@intracraft/sim-des';
import { describe, expect, it } from 'vitest';
import { LAYOUT_FILES, runFlowScenario } from './flow';

const REGISTRY = createBuiltInRegistry();

describe('layout files', () => {
  it.each(Object.keys(LAYOUT_FILES))(
    '%s is a valid project whose objects are all connected',
    (name) => {
      const file = readProjectFile(JSON.stringify(LAYOUT_FILES[name]));
      if (!file.ok) throw new Error(file.error);
      const layout = {
        ...file.project.layout,
        connections: deriveConnections(file.project.layout, REGISTRY),
      };
      expect(layout.connections).toHaveLength(layout.instances.length - 1);
      expect(buildFlowNetwork(layout, REGISTRY).issues).toEqual([]);
    },
  );

  it('reports a missing statistic instead of returning a number', () => {
    expect(() =>
      runFlowScenario({ layout: 'flow-t1', seconds: 1, instanceId: 'nothing', statistic: 'x' }),
    ).toThrow();
  });
});
