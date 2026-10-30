import { rotateY } from '@intracraft/library';
import { describe, expect, it } from 'vitest';
import { frameAxis, toLocal, toWorld } from './frame';
import { profileHeight } from './station';

describe('deck frames', () => {
  it('turn like the instance frame of the editor (library rotateY)', () => {
    for (const yaw of [0, 0.3, Math.PI / 2, Math.PI, -2]) {
      const frame = { origin: { x: 1, y: 0.6, z: -2 }, yaw, slope: 0 };
      const world = toWorld(frame, { x: 2, y: 0, z: 0.5 });
      const expected = rotateY({ x: 2, z: 0.5 }, yaw);
      expect(world.x).toBeCloseTo(1 + expected.x, 12);
      expect(world.y).toBeCloseTo(0.6, 12);
      expect(world.z).toBeCloseTo(-2 + expected.z, 12);
    }
  });

  it('fall along travel with the slope and convert back exactly', () => {
    const frame = { origin: { x: 0, y: 0.34, z: 0 }, yaw: 0.7, slope: 0.04 };
    const end = toWorld(frame, { x: 6 / Math.cos(Math.atan(0.04)), y: 0, z: 0 });
    expect(end.y).toBeCloseTo(0.34 - 0.24, 12);
    expect(frameAxis(frame, 'z').y).toBeCloseTo(0, 12);
    const local = toLocal(frame, toWorld(frame, { x: 1.2, y: 0.1, z: -0.3 }));
    expect(local.x).toBeCloseTo(1.2, 12);
    expect(local.y).toBeCloseTo(0.1, 12);
    expect(local.z).toBeCloseTo(-0.3, 12);
  });
});

describe('vertical profile of a lift station (ADR-0013 §3)', () => {
  it('reaches 4 m in 4 / 0.5 + 0.5 / 0.5 = 9 s, half way at the middle', () => {
    const move = { from: 0.6, to: 4.6, start: 10, duration: 9, speed: 0.5, accel: 0.5 };
    expect(profileHeight(move, 10)).toBeCloseTo(0.6, 12);
    expect(profileHeight(move, 11)).toBeCloseTo(0.6 + 0.25, 12);
    expect(profileHeight(move, 14.5)).toBeCloseTo(2.6, 12);
    expect(profileHeight(move, 19)).toBeCloseTo(4.6, 12);
    expect(profileHeight(move, 30)).toBeCloseTo(4.6, 12);
  });

  it('goes down without reaching full speed on a stroke shorter than v²/a (10 mm < 24.5 mm)', () => {
    const move = {
      from: 0.6,
      to: 0.59,
      start: 0,
      duration: 2 * Math.sqrt(0.01 / 0.2),
      speed: 0.07,
      accel: 0.2,
    };
    expect(profileHeight(move, move.duration / 2)).toBeCloseTo(0.595, 12);
    expect(profileHeight(move, move.duration)).toBeCloseTo(0.59, 12);
  });
});
