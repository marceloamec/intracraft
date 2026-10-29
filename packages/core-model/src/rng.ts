// Seeded pseudo-random generator (PCG32, XSH-RR variant). Engines must never call Math.random().

const MULTIPLIER = 6364136223846793005n;
const MASK_64 = (1n << 64n) - 1n;
const UINT32_RANGE = 4294967296;

export interface Rng {
  /** Uniform integer in [0, 2^32). */
  nextUint32(): number;
  /** Uniform float in [0, 1). */
  next(): number;
  /** Uniform float in [min, max). */
  uniform(min: number, max: number): number;
}

export function createRng(seed: number | bigint, stream: number | bigint = 54n): Rng {
  const increment = ((BigInt(stream) << 1n) | 1n) & MASK_64;
  let state = 0n;

  const nextUint32 = (): number => {
    const old = state;
    state = (old * MULTIPLIER + increment) & MASK_64;
    const xorShifted = Number((((old >> 18n) ^ old) >> 27n) & 0xffffffffn);
    const rot = Number(old >> 59n);
    return ((xorShifted >>> rot) | (xorShifted << (-rot & 31))) >>> 0;
  };

  nextUint32();
  state = (state + (BigInt(seed) & MASK_64)) & MASK_64;
  nextUint32();

  const next = (): number => nextUint32() / UINT32_RANGE;
  return {
    nextUint32,
    next,
    uniform: (min, max) => min + (max - min) * next(),
  };
}
