import { createBuiltInRegistry } from '@intracraft/library';
import { describe, expect, it } from 'vitest';
import { Session } from './session';

const registry = createBuiltInRegistry();

describe('layout.addInstance', () => {
  it('places a conveyor with all default parameters stored and a readable id', () => {
    const session = new Session(registry);
    const result = session.execute('layout.addInstance', { definitionId: 'conveyor.roller' }, 'user');
    expect(result).toMatchObject({ ok: true, instanceId: 'roller1' });
    const [instance] = session.layout.instances;
    expect(instance?.params['rollerPitch']).toBe(0.2063);
    expect(instance?.name).toBe('roller1');
    expect(instance?.attributes).toEqual({});
  });

  it('numbers instances per type, deterministically', () => {
    const session = new Session(registry);
    const ids = ['conveyor.roller', 'conveyor.roller', 'conveyor.chain_turntable'].map(
      (definitionId) => session.execute('layout.addInstance', { definitionId }, 'user').instanceId,
    );
    expect(ids).toEqual(['roller1', 'roller2', 'chain_turntable1']);
  });

  it('rejects an unknown definition and a parameter outside its range, changing nothing', () => {
    const session = new Session(registry);
    expect(session.execute('layout.addInstance', { definitionId: 'conveyor.spiral' }, 'user').ok).toBe(false);
    const result = session.execute(
      'layout.addInstance',
      { definitionId: 'conveyor.roller', params: { length: 50 } },
      'user',
    );
    expect(result.ok).toBe(false);
    expect(result.issues[0]?.rule).toBe('paramOutOfRange');
    expect(session.layout.instances).toEqual([]);
  });

  it('connects two conveyors placed end to end', () => {
    const session = new Session(registry);
    session.execute('layout.addInstance', { definitionId: 'conveyor.roller' }, 'user');
    session.execute('layout.addInstance', { definitionId: 'conveyor.roller', position: { x: 3.02, y: 0, z: 0 } }, 'user');
    expect(session.layout.connections).toMatchObject([
      { from: { instanceId: 'roller1', portId: 'B' }, to: { instanceId: 'roller2', portId: 'A' } },
    ]);
  });
});

describe('layout.setParams', () => {
  it('accepts a roller pitch from the list and rejects one that is not', () => {
    const session = withRoller();
    expect(session.execute('layout.setParams', { instanceId: 'roller1', params: { rollerPitch: 0.133 } }, 'user').ok).toBe(
      true,
    );
    const result = session.execute('layout.setParams', { instanceId: 'roller1', params: { rollerPitch: 0.15 } }, 'user');
    expect(result.issues[0]?.rule).toBe('paramNotAChoice');
    expect(session.layout.instances[0]?.params['rollerPitch']).toBe(0.133);
  });

  it('rejects wrong types, unknown keys, unknown enum options and derived values', () => {
    const session = withRoller();
    const rules = [
      { length: 'long' },
      { colour: 3 },
      { accumulationMode: 'magnetic' },
      { rollerCount: 20 },
    ].map((params) => session.execute('layout.setParams', { instanceId: 'roller1', params }, 'user').issues[0]?.rule);
    expect(rules).toEqual(['paramWrongType', 'unknownParam', 'paramNotAnOption', 'paramReadOnly']);
  });

  it('accepts the change but warns that a crosswise EUR pallet does not fit the width', () => {
    const session = withRoller();
    const result = session.execute(
      'layout.setParams',
      { instanceId: 'roller1', params: { palletSideAlongTravel: 'short' } },
      'user',
    );
    expect(result.ok).toBe(true);
    expect(result.issues.map((i) => i.rule)).toEqual(['usableWidthFitsPallet']);
  });
});

describe('layout.move, layout.setAttributes, layout.remove, layout.clear', () => {
  it('moves and rotates an instance, keeping the yaw within one turn', () => {
    const session = withRoller();
    session.execute('layout.move', { instanceId: 'roller1', position: { x: 1, y: 0, z: 2 }, yaw: (5 * Math.PI) / 2 }, 'user');
    expect(session.layout.instances[0]?.position).toEqual({ x: 1, y: 0, z: 2 });
    expect(session.layout.instances[0]?.yaw).toBeCloseTo(Math.PI / 2, 12);
  });

  it('changes parameters together with the move in one undo step, or rejects both', () => {
    const session = withRoller();
    session.execute('layout.move', { instanceId: 'roller1', position: { x: 1, y: 0, z: 0 }, yaw: 0, params: { length: 4 } }, 'user');
    expect(session.layout.instances[0]).toMatchObject({ position: { x: 1 }, params: { length: 4 } });
    const rejected = session.execute('layout.move', { instanceId: 'roller1', position: { x: 9, y: 0, z: 0 }, yaw: 0, params: { length: 99 } }, 'user');
    expect(rejected.ok).toBe(false);
    expect(session.layout.instances[0]?.position.x).toBe(1);
    session.undo('user');
    expect(session.layout.instances[0]).toMatchObject({ position: { x: 0 }, params: { length: 3 } });
  });

  it('renames and stores free attributes', () => {
    const session = withRoller();
    session.execute(
      'layout.setAttributes',
      { instanceId: 'roller1', name: 'Infeed', attributes: { zone: 'A', line: 3 } },
      'user',
    );
    expect(session.layout.instances[0]).toMatchObject({ name: 'Infeed', attributes: { zone: 'A', line: 3 } });
  });

  it('removes an instance and its connections', () => {
    const session = withRoller();
    session.execute('layout.addInstance', { definitionId: 'conveyor.roller', position: { x: 3.02, y: 0, z: 0 } }, 'user');
    session.execute('layout.remove', { instanceId: 'roller2' }, 'user');
    expect(session.layout.instances.map((i) => i.id)).toEqual(['roller1']);
    expect(session.layout.connections).toEqual([]);
  });

  it('rejects commands on an unknown instance', () => {
    const session = withRoller();
    const result = session.execute('layout.remove', { instanceId: 'roller9' }, 'user');
    expect(result).toMatchObject({ ok: false, issues: [{ rule: 'unknownInstance' }] });
  });

  it('clears the layout', () => {
    const session = withRoller();
    session.execute('layout.clear', {}, 'user');
    expect(session.layout).toEqual({ instances: [], connections: [] });
  });
});

describe('layout.load', () => {
  it('replaces the layout, fills new parameters with defaults and derives the connections', () => {
    const source = withRoller();
    source.execute('layout.addInstance', { definitionId: 'conveyor.roller', position: { x: 3.02, y: 0, z: 0 } }, 'user');
    const saved = structuredClone(source.layout);
    delete saved.instances[0]?.params['minGap'];
    saved.connections = [];

    const session = withRoller();
    const result = session.execute('layout.load', { layout: saved }, 'user');
    expect(result.ok).toBe(true);
    expect(session.layout.instances[0]?.params['minGap']).toBe(0.15);
    expect(session.layout.connections).toHaveLength(1);
    session.undo('user');
    expect(session.layout.instances.map((i) => i.id)).toEqual(['roller1']);
  });

  it('rejects the whole file on an unknown component, a duplicate id or an invalid parameter', () => {
    const session = withRoller();
    const roller = session.layout.instances[0];
    if (roller === undefined) throw new Error('no roller');
    const bad = {
      instances: [
        roller,
        { ...roller },
        { ...roller, id: 'x1', definitionId: 'conveyor.unknown' },
        { ...roller, id: 'x2', params: { length: 99 } },
      ],
      connections: [],
    };
    const result = session.execute('layout.load', { layout: bad }, 'user');
    expect(result.ok).toBe(false);
    expect(result.issues.map((i) => i.rule)).toEqual(['duplicateInstanceId', 'unknownDefinition', 'paramOutOfRange']);
    expect(session.layout.instances).toHaveLength(1);
  });
});

describe('control.setRules', () => {
  const stopRule = {
    id: 'stopAtSensor',
    when: [{ signal: 'photoeye1.detected', edge: 'rising' as const }],
    then: [{ signal: 'roller1.run', value: false }],
  };

  it('stores the rules with the layout, undoably, and warns about unknown objects', () => {
    const session = withRoller();
    const result = session.execute('control.setRules', { rules: [stopRule] }, 'user');
    expect(result.ok).toBe(true);
    expect(result.issues.map((i) => i.message)).toEqual([expect.stringContaining('photoeye1.detected')]);
    expect(session.layout.rules).toEqual([stopRule]);
    session.undo('user');
    expect(session.layout.rules).toBeUndefined();
  });

  it('rejects duplicate rule ids and keeps the rules through layout.load', () => {
    const session = withRoller();
    expect(session.execute('control.setRules', { rules: [stopRule, stopRule] }, 'user').ok).toBe(false);
    session.execute('control.setRules', { rules: [stopRule] }, 'user');
    session.execute('layout.load', { layout: structuredClone(session.layout) }, 'user');
    expect(session.layout.rules).toEqual([stopRule]);
  });
});

describe('undo, redo and the command log', () => {
  it('undo and redo give back exactly the same layouts', () => {
    const session = withRoller();
    const before = session.layout;
    session.execute('layout.setParams', { instanceId: 'roller1', params: { length: 4 } }, 'user');
    const after = session.layout;
    expect(session.undo('user')).toBe(true);
    expect(session.layout).toEqual(before);
    expect(session.redo('user')).toBe(true);
    expect(session.layout).toEqual(after);
  });

  it('a new command clears the redo history; nothing to undo on a fresh session', () => {
    const session = withRoller();
    session.undo('user');
    session.execute('layout.addInstance', { definitionId: 'conveyor.chain' }, 'user');
    expect(session.canRedo).toBe(false);
    expect(new Session(registry).undo('user')).toBe(false);
  });

  it('a rejected command is logged but cannot be undone', () => {
    const session = new Session(registry);
    session.execute('layout.remove', { instanceId: 'x' }, 'user');
    expect(session.canUndo).toBe(false);
    expect(session.log[0]).toMatchObject({ seq: 1, command: 'layout.remove', ok: false });
  });

  it('records who did what', () => {
    const session = new Session(registry);
    session.execute('layout.addInstance', { definitionId: 'conveyor.roller' }, 'mcp:test-client');
    session.undo('chat');
    expect(session.log.map((e) => `${e.actor} ${e.command}`)).toEqual([
      'mcp:test-client layout.addInstance',
      'chat edit.undo',
    ]);
  });

  it('replaying the log on a new session rebuilds the same layout (determinism)', () => {
    const session = withRoller();
    session.execute('layout.addInstance', { definitionId: 'conveyor.transfer_rc', position: { x: 3.02, y: 0, z: 0 } }, 'user');
    session.execute('layout.setAttributes', { instanceId: 'transfer_rc1', attributes: { zone: 'B' } }, 'user');
    const replay = new Session(registry);
    for (const entry of session.log) replay.replay(entry);
    expect(replay.layout).toEqual(session.layout);
  });
});

function withRoller(): Session {
  const session = new Session(registry);
  session.execute('layout.addInstance', { definitionId: 'conveyor.roller' }, 'user');
  return session;
}
