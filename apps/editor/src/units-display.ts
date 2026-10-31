// Conversion between stored SI values and the units shown in the editor (mm, ms, degrees, %).
import { degToRad, mToMm, mmToM, radToDeg } from '@intracraft/core-model';

const MAX_DECIMALS = 4;
const MM_DECIMALS = 2;

const TO_DISPLAY: Record<string, (value: number) => number> = {
  'm>mm': mToMm,
  's>ms': (s) => s * 1000,
  'rad>°': radToDeg,
  'rad/s>°/s': radToDeg,
  'rad/s²>°/s²': radToDeg,
  'ratio>%': (ratio) => ratio * 100,
};

const FROM_DISPLAY: Record<string, (value: number) => number> = {
  'm>mm': mmToM,
  's>ms': (ms) => ms / 1000,
  'rad>°': degToRad,
  'rad/s>°/s': degToRad,
  'rad/s²>°/s²': degToRad,
  'ratio>%': (percent) => percent / 100,
};

export function toDisplay(value: number, unit?: string, displayUnit?: string): number {
  return TO_DISPLAY[`${unit}>${displayUnit}`]?.(value) ?? value;
}

export function fromDisplay(value: number, unit?: string, displayUnit?: string): number {
  return FROM_DISPLAY[`${unit}>${displayUnit}`]?.(value) ?? value;
}

/** Short text for a displayed number: at most 4 decimals (2 for mm), no trailing zeros. */
export function formatNumber(value: number, displayUnit?: string): string {
  return String(Number(value.toFixed(displayUnit === 'mm' ? MM_DECIMALS : MAX_DECIMALS)));
}
