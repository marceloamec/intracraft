import { describe, expect, it } from 'vitest';
import { projectFileName, readProjectFile, writeProjectFile } from './project-file';
import type { Layout } from './schema';

const LAYOUT: Layout = {
  instances: [
    {
      id: 'source1',
      definitionId: 'flow.source',
      definitionVersion: '0.1.0',
      name: 'Source 1',
      position: { x: 0, y: 0, z: 0 },
      yaw: 0,
      params: { interval: 20 },
      attributes: { zone: 'A' },
    },
  ],
  connections: [],
};

describe('project file', () => {
  it('reads back exactly what it wrote', () => {
    const text = writeProjectFile('Line 1', 42, LAYOUT);
    const result = readProjectFile(text);
    expect(result).toEqual({ ok: true, project: { schemaVersion: 1, name: 'Line 1', seed: 42, layout: LAYOUT } });
    expect(text.endsWith('\n')).toBe(true);
  });

  it('fills defaults of optional fields (attributes)', () => {
    const withoutAttributes: Record<string, unknown> = { ...LAYOUT.instances[0] };
    delete withoutAttributes['attributes'];
    const text = JSON.stringify({ schemaVersion: 1, name: 'x', seed: 1, layout: { instances: [withoutAttributes], connections: [] } });
    const result = readProjectFile(text);
    expect(result.ok && result.project.layout.instances[0]?.attributes).toEqual({});
  });

  it('rejects text that is not JSON, not a project, or from a newer version', () => {
    expect(readProjectFile('{').ok).toBe(false);
    expect(readProjectFile('[]').ok).toBe(false);
    expect(readProjectFile('{"name":"x"}')).toEqual({ ok: false, error: expect.stringContaining('schemaVersion') });
    const newer = readProjectFile('{"schemaVersion": 99, "name": "x", "seed": 1, "layout": {}}');
    expect(newer).toEqual({ ok: false, error: expect.stringContaining('version 99') });
  });

  it('says where an invalid file is wrong', () => {
    const result = readProjectFile('{"schemaVersion": 1, "name": "x", "seed": -1, "layout": {"instances": [], "connections": []}}');
    expect(result).toEqual({ ok: false, error: expect.stringContaining('seed') });
  });

  it('makes safe file names', () => {
    expect(projectFileName('Línea 1: entrada/salida')).toBe('Línea-1-entradasalida.intracraft.json');
    expect(projectFileName('  ')).toBe('layout.intracraft.json');
  });
});
