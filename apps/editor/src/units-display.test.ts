import { describe, expect, it } from 'vitest';
import { formatNumber, fromDisplay, toDisplay } from './units-display';

describe('display units', () => {
  it('shows metres as millimetres and back', () => {
    expect(toDisplay(0.2063, 'm', 'mm')).toBeCloseTo(206.3, 9);
    expect(fromDisplay(206.3, 'm', 'mm')).toBeCloseTo(0.2063, 12);
  });

  it('shows seconds as milliseconds and back', () => {
    expect(toDisplay(0.002, 's', 'ms')).toBeCloseTo(2, 12);
    expect(fromDisplay(2000, 's', 'ms')).toBe(2);
  });

  it('shows radians, rad/s and rad/s² as degrees', () => {
    expect(toDisplay(Math.PI / 2, 'rad', '°')).toBeCloseTo(90, 9);
    expect(toDisplay((15 * Math.PI) / 180, 'rad/s', '°/s')).toBeCloseTo(15, 9);
    expect(fromDisplay(30, 'rad/s²', '°/s²')).toBeCloseTo(Math.PI / 6, 12);
  });

  it('shows a slope ratio as a percentage and back', () => {
    expect(toDisplay(0.04, 'ratio', '%')).toBeCloseTo(4, 12);
    expect(fromDisplay(4, 'ratio', '%')).toBeCloseTo(0.04, 12);
  });

  it('leaves values unchanged when the display unit equals the stored unit', () => {
    expect(toDisplay(0.25, 'm/s', 'm/s')).toBe(0.25);
    expect(fromDisplay(1000, 'kg', undefined)).toBe(1000);
  });

  it('formats without floating-point noise', () => {
    expect(formatNumber(toDisplay(0.1 + 0.2, 'm', 'mm'))).toBe('300');
    expect(formatNumber(206.30000000000001)).toBe('206.3');
    expect(formatNumber(0.35)).toBe('0.35');
    expect(formatNumber(971.9544, 'mm')).toBe('971.95');
  });
});
