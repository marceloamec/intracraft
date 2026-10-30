import { describe, expect, it } from 'vitest';
import { validateInstance } from './instance-validation';
import { derivedValues } from './params';
import { FRICTION_PAIRS, PALLET_TYPES } from './reference-data';
import { createBuiltInRegistry, defaultParams } from './registry';
import {
  chainFitsBetweenRollers,
  min3RollersUnderPallet,
  usableWidthFitsPallet,
} from './validation-rules';

const DEFINITION_IDS = createBuiltInRegistry()
  .list()
  .map((d) => d.id);

describe('built-in registry', () => {
  it('loads and validates the roller conveyor definition', () => {
    const roller = createBuiltInRegistry().get('conveyor.roller');
    expect(roller.name.de).toBe('Rollenförderer');
    expect(roller.ports.map((p) => p.id)).toEqual(['A', 'B']);
  });

  it('contains the five MVP conveyors, the source, the sink, the process station and the photocell', () => {
    expect(createBuiltInRegistry().list().map((d) => d.id)).toEqual([
      'conveyor.roller',
      'conveyor.chain',
      'conveyor.chain_turntable',
      'conveyor.roller_turntable',
      'conveyor.transfer_rc',
      'flow.source',
      'flow.sink',
      'flow.process_station',
      'sensor.photoeye',
    ]);
  });

  it('offers every library pallet type in the source', () => {
    const palletType = createBuiltInRegistry()
      .get('flow.source')
      .parameters.find((p) => p.key === 'palletType');
    expect(palletType?.options).toEqual(PALLET_TYPES.map((p) => p.id));
  });

  describe.each(DEFINITION_IDS)('%s', (id) => {
    const definition = createBuiltInRegistry().get(id);

    it('has an English label and description for every parameter (translations optional)', () => {
      for (const p of definition.parameters) {
        expect(p.label.en, p.key).toBeTruthy();
        expect(p.description.en, p.key).toBeTruthy();
      }
    });

    it('has unique parameter keys and unique port ids', () => {
      const keys = definition.parameters.map((p) => p.key);
      expect(new Set(keys).size).toBe(keys.length);
      const ports = definition.ports.map((p) => p.id);
      expect(new Set(ports).size).toBe(ports.length);
    });

    it('keeps every default inside its [min, max] range and its list of choices', () => {
      for (const p of definition.parameters) {
        if (typeof p.default !== 'number') continue;
        if (p.min !== undefined) expect(p.default, p.key).toBeGreaterThanOrEqual(p.min);
        if (p.max !== undefined) expect(p.default, p.key).toBeLessThanOrEqual(p.max);
        if (p.choices !== undefined) expect(p.choices, p.key).toContain(p.default);
      }
    });

    it('gives every enum default one of its options', () => {
      for (const p of definition.parameters) {
        if (p.type === 'enum') expect(p.options, p.key).toContain(p.default);
      }
    });

    it('passes its own validation rules with the default values', () => {
      expect(validateInstance(definition, {})).toEqual([]);
    });
  });

  it('defaults to Ø89 mm rollers at 206.3 mm pitch and no accumulation', () => {
    const params = defaultParams(createBuiltInRegistry().get('conveyor.roller'));
    expect(params['rollerDiameter']).toBe(0.089);
    expect(params['rollerPitch']).toBe(0.2063);
    expect(params['accumulationMode']).toBe('none');
  });

  it('excludes derived parameters from the defaults', () => {
    const params = defaultParams(createBuiltInRegistry().get('conveyor.roller'));
    expect(params['length']).toBe(3);
    expect(params['rollerCount']).toBeUndefined();
  });
});

describe('derived values', () => {
  it('counts 14 rollers on a 3 m conveyor at 206.3 mm pitch and 22 at 133 mm', () => {
    const roller = createBuiltInRegistry().get('conveyor.roller');
    expect(derivedValues(roller, defaultParams(roller))).toEqual({ rollerCount: 14 });
    expect(derivedValues(roller, { ...defaultParams(roller), rollerPitch: 0.133 })).toEqual({ rollerCount: 22 });
  });

  it('gives the turntable a swept radius of 972 mm with the defaults (S03)', () => {
    const turntable = createBuiltInRegistry().get('conveyor.chain_turntable');
    expect(derivedValues(turntable, defaultParams(turntable))['sweptRadius']).toBeCloseTo(0.972, 3);
  });
});

describe('validateInstance', () => {
  it('flags a chain conveyor too narrow for a transverse EUR pallet', () => {
    const chain = createBuiltInRegistry().get('conveyor.chain');
    expect(validateInstance(chain, { usableWidth: 1.0 }).map((i) => i.rule)).toEqual(['usableWidthFitsPallet']);
  });

  it('flags a transfer whose transfer chains do not fit between the rollers', () => {
    const transfer = createBuiltInRegistry().get('conveyor.transfer_rc');
    const issues = validateInstance(transfer, { rollerPitch: 0.133, rollerDiameter: 0.089, transferChainWidth: 0.08 });
    expect(issues.map((i) => i.rule)).toEqual(['chainFitsBetweenRollers']);
  });
});

describe('reference data', () => {
  it('includes EUR, 1200 × 1000 and GMA pallets', () => {
    expect(PALLET_TYPES.map((p) => p.id)).toEqual(['eur', 'iso1200x1000', 'gma']);
  });

  it('describes both travel orientations with bands that stack up to the pallet height', () => {
    for (const pallet of PALLET_TYPES) {
      for (const [side, bands] of Object.entries(pallet.occupancy)) {
        const extent = side === 'alongLength' ? pallet.length : pallet.width;
        expect(bands[0]?.fromHeight, `${pallet.id} ${side}`).toBe(0);
        expect(bands.at(-1)?.toHeight, `${pallet.id} ${side}`).toBe(pallet.height);
        for (const band of bands) {
          for (const [from, to] of band.solid) {
            expect(from).toBeGreaterThanOrEqual(0);
            expect(to).toBeLessThanOrEqual(extent + 1e-9);
            expect(to).toBeGreaterThan(from);
          }
        }
      }
    }
  });

  it('keeps each friction default inside its usual range', () => {
    for (const pair of FRICTION_PAIRS) {
      expect(pair.mu).toBeGreaterThanOrEqual(pair.range[0]);
      expect(pair.mu).toBeLessThanOrEqual(pair.range[1]);
    }
  });
});

describe('validation rules', () => {
  it('accepts the default roller pitch (206.3 mm) under a 1200 mm side', () => {
    expect(min3RollersUnderPallet(0.2063, 1.2)).toEqual([]);
  });

  it('flags a pitch that leaves fewer than 3 rollers', () => {
    expect(min3RollersUnderPallet(0.25, 0.7)).toHaveLength(1);
  });

  it('checks usable width against pallet width and guide clearance', () => {
    expect(usableWidthFitsPallet(1.0, 0.8, 0.025)).toEqual([]);
    expect(usableWidthFitsPallet(0.82, 0.8, 0.025)).toHaveLength(1);
  });

  it('accepts a 40 mm chain between Ø89 rollers at 206.3 mm pitch (gap 117.3 mm)', () => {
    expect(chainFitsBetweenRollers(0.2063, 0.089, 0.04)).toEqual([]);
    expect(chainFitsBetweenRollers(0.133, 0.076, 0.04)).toEqual([]);
    expect(chainFitsBetweenRollers(0.1, 0.076, 0.04)).toHaveLength(1);
  });
});
