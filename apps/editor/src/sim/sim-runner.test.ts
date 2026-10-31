import type { Instance, Layout } from '@intracraft/core-model';
import { createBuiltInRegistry, deriveConnections, instanceParams } from '@intracraft/library';
import { describe, expect, it, vi } from 'vitest';
import type { FromWorker } from './protocol';
import { SimRunner } from './sim-runner';

const REGISTRY = createBuiltInRegistry();
const BOX = 'asset.boxes.box_cardboard_a01';

describe('simulation runner', () => {
  it('does not advance until played, then follows the wall clock times the speed', () => {
    const runner = new SimRunner(() => 0);
    expect(
      runner.handle({ type: 'load', layout: sourceRollerSink(), seed: 1 })?.snapshot.time,
    ).toBe(0);
    expect(runner.tick(1000)).toBeUndefined();

    runner.handle({ type: 'setSpeed', speed: 10 });
    runner.handle({ type: 'play' });
    expect(runner.tick(500)?.snapshot.time).toBe(5_000_000);
  });

  it('steps to the next event and pauses', () => {
    const runner = new SimRunner(() => 0);
    runner.handle({ type: 'load', layout: sourceRollerSink(), seed: 1 });
    const first = runner.handle({ type: 'step' }); // the source creates the first pallet at t = 0
    expect(first?.playing).toBe(false);
    expect(first?.snapshot.pallets).toHaveLength(1);
    expect(runner.handle({ type: 'step' })?.snapshot.time).toBe(500_000); // end of the start ramp
  });

  it('runs as fast as the time budget allows at max speed', () => {
    let now = 0;
    const runner = new SimRunner(() => (now += 1));
    runner.handle({ type: 'load', layout: sourceRollerSink(), seed: 1 });
    runner.handle({ type: 'setSpeed', speed: 'max' });
    runner.handle({ type: 'play' });
    const frame = runner.tick(33);
    expect(frame?.snapshot.time).toBeGreaterThanOrEqual(120_000_000);
    expect(frame?.snapshot.time).toBeLessThanOrEqual(3600_000_000);
  });

  it('drops a catalogue object into a running 3D run without resetting it, and logs the command', async () => {
    const posted: FromWorker[] = [];
    const runner = new SimRunner(
      () => 0,
      (message) => posted.push(message),
    );
    runner.handle({ type: 'load', layout: sourceRollerSink(), seed: 1, mode: 'physics' });
    await vi.waitFor(() => expect(posted).toHaveLength(1));
    runner.handle({ type: 'play' });
    runner.tick(500);
    const input = {
      asset: BOX,
      placement: 'drop',
      dropHeight: 1,
      position: { x: 5, y: 0, z: 2 },
      yaw: 0,
    };
    const frame = runner.handle({ type: 'spawn', input });
    expect(frame?.snapshot.time).toBe(500_000);
    expect(frame?.snapshot.objects?.['live1']?.asset).toBe(BOX);
    const later = runner.tick(1500);
    expect(later?.snapshot.time).toBe(2_000_000);
    expect(later?.snapshot.objects?.['live1']?.pose.position.y).toBeLessThan(0.3);
    expect(runner.log).toEqual([
      { time: 500_000, actor: 'user', command: 'sim.spawnObject', input, objectId: 'live1' },
    ]);
  });

  it('rejects an invalid object, and any object in a 2D run, with a message', async () => {
    const posted: FromWorker[] = [];
    const runner = new SimRunner(
      () => 0,
      (message) => posted.push(message),
    );
    const object = {
      asset: BOX,
      placement: 'drop',
      dropHeight: 1,
      position: { x: 0, y: 0, z: 0 },
      yaw: 0,
    };
    runner.handle({ type: 'load', layout: sourceRollerSink(), seed: 1 });
    expect(runner.handle({ type: 'spawn', input: object })).toBeUndefined();
    runner.handle({ type: 'load', layout: sourceRollerSink(), seed: 1, mode: 'physics' });
    await vi.waitFor(() => expect(posted.some((m) => m.type === 'frame')).toBe(true));
    expect(runner.handle({ type: 'spawn', input: { ...object, dropHeight: 50 } })).toBeUndefined();
    expect(
      posted.filter((m) => m.type === 'error').map((m) => (m.type === 'error' ? m.message : '')),
    ).toEqual([expect.stringMatching(/3D/), expect.stringMatching(/dropHeight/)]);
    expect(runner.log).toEqual([]);
  });

  it('loads the 3D run in the background, then follows the clock with the real pallet poses', async () => {
    const posted: FromWorker[] = [];
    const runner = new SimRunner(
      () => 0,
      (message) => posted.push(message),
    );
    expect(
      runner.handle({ type: 'load', layout: sourceRollerSink(), seed: 1, mode: 'physics' }),
    ).toBeUndefined();
    await vi.waitFor(() => expect(posted).toHaveLength(1));
    expect(posted[0]?.type).toBe('frame');

    runner.handle({ type: 'play' });
    const frame = runner.tick(500);
    expect(frame?.snapshot.time).toBe(500_000);
    const [pallet] = frame?.snapshot.pallets ?? [];
    expect(pallet?.carrier).toBe('roller1');
    expect(frame?.snapshot.poses?.[pallet?.id ?? 0]?.position.y).toBeCloseTo(0.6, 2);
  });

  it('reports the warnings of the layout once, with the first frame', () => {
    const runner = new SimRunner(() => 0);
    const layout = sourceRollerSink();
    const open = {
      ...layout,
      instances: layout.instances.slice(0, 2),
      connections: layout.connections.slice(0, 1),
    };
    expect(
      runner.handle({ type: 'load', layout: open, seed: 1 })?.issues?.map((i) => i.rule),
    ).toEqual(['openEnd']);
    runner.handle({ type: 'play' });
    expect(runner.tick(33)?.issues).toBeUndefined();
  });
});

function sourceRollerSink(): Layout {
  const parts: [string, string, number][] = [
    ['source1', 'flow.source', 0],
    ['roller1', 'conveyor.roller', 0.42],
    ['sink1', 'flow.sink', 3.44],
  ];
  const instances: Instance[] = parts.map(([id, definitionId, x]) => ({
    id,
    definitionId,
    definitionVersion: '0.1.0',
    name: id,
    position: { x, y: 0, z: 0 },
    yaw: 0,
    params: instanceParams(REGISTRY.get(definitionId), {}),
    attributes: {},
  }));
  const layout: Layout = { instances, connections: [] };
  return { ...layout, connections: deriveConnections(layout, REGISTRY) };
}
