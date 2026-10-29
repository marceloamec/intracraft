import { describe, expect, it } from 'vitest';
import { SCHEMA_VERSION, localize, projectSchema } from './schema';

describe('project schema', () => {
  it('accepts a minimal Source → Sink project', () => {
    const project = {
      schemaVersion: SCHEMA_VERSION,
      name: 'Source to Sink',
      seed: 1,
      layout: {
        instances: [
          {
            id: 'src1',
            definitionId: 'flow.source',
            definitionVersion: '0.1.0',
            name: 'Source 1',
            position: { x: 0, y: 0, z: 0 },
            yaw: 0,
            params: { interval: 20 },
          },
        ],
        connections: [],
      },
    };
    expect(projectSchema.parse(project).name).toBe('Source to Sink');
  });

  it('falls back to English when a translation is missing', () => {
    expect(localize({ en: 'Roller conveyor', es: 'Mesa de rodillos' }, 'es')).toBe('Mesa de rodillos');
    expect(localize({ en: 'Roller conveyor' }, 'de')).toBe('Roller conveyor');
  });

  it('rejects a project without schemaVersion', () => {
    expect(() => projectSchema.parse({ name: 'x', seed: 1, layout: {} })).toThrow();
  });
});
