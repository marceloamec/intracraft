import { builtInCatalog } from '@intracraft/asset-catalog';
import { describe, expect, it } from 'vitest';
import { validateInstance } from './instance-validation';
import { derivedValues } from './params';
import { CONTACT_MATERIALS, CONTACT_PAIRS, PALLET_TYPES, contactPair } from './reference-data';
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

  it('contains the five MVP conveyors, the lifts, the source, the sink, the process station, the photocell and the scenery object', () => {
    expect(
      createBuiltInRegistry()
        .list()
        .map((d) => d.id),
    ).toEqual([
      'conveyor.roller',
      'conveyor.chain',
      'conveyor.chain_turntable',
      'conveyor.roller_turntable',
      'conveyor.transfer_rc',
      'conveyor.lift_table_manual',
      'conveyor.scissor_lift',
      'conveyor.elevator',
      'conveyor.gravity',
      'flow.source',
      'flow.sink',
      'flow.process_station',
      'sensor.photoeye',
      'scenery.asset',
    ]);
  });

  it('offers every catalogue model in the scenery object, named as in the catalogue (S17)', () => {
    const asset = param('scenery.asset', 'asset');
    expect(asset?.options).toEqual(builtInCatalog.list().map((a) => a.id));
    for (const a of builtInCatalog.list()) expect(asset?.optionLabels?.[a.id]?.en).toBe(a.name.en);
  });

  it('offers every contact material in the scenery object, or the material of the model (auto)', () => {
    expect(param('scenery.asset', 'material')?.options).toEqual([
      'auto',
      ...CONTACT_MATERIALS.map((m) => m.id),
    ]);
  });

  it('offers catalogue pallets as pallet models and boxes, containers and drums as loads (S09 §1.1)', () => {
    const pallets = builtInCatalog.byCategory('pallets').map((a) => a.id);
    expect(param('flow.source', 'palletModel')?.options).toEqual(['auto', 'none', ...pallets]);
    const loads = ['boxes', 'containers', 'drums'] as const;
    expect(param('flow.source', 'loadAsset')?.options).toEqual(
      loads.flatMap((c) => builtInCatalog.byCategory(c).map((a) => a.id)),
    );
    expect(param('flow.source', 'loadKind')?.options).toEqual(['block', 'none', 'asset']);
  });

  it('keeps the current source behaviour by default: pallet model of the type and a load block', () => {
    const params = defaultParams(createBuiltInRegistry().get('flow.source'));
    expect(params).toMatchObject({ palletModel: 'auto', loadKind: 'block' });
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

    it('labels every option of a parameter that declares option labels', () => {
      for (const p of definition.parameters) {
        if (p.optionLabels === undefined) continue;
        for (const option of p.options ?? [])
          expect(p.optionLabels[option]?.en, `${p.key} ${option}`).toBeTruthy();
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
  it('derives the gravity line entry height: 0.1 m + 4 % × 6 m = 0.34 m (S15)', () => {
    const gravity = createBuiltInRegistry().get('conveyor.gravity');
    expect(derivedValues(gravity, defaultParams(gravity))['entryHeight']).toBeCloseTo(0.34, 12);
  });

  it('counts 14 rollers on a 3 m conveyor at 206.3 mm pitch and 22 at 133 mm', () => {
    const roller = createBuiltInRegistry().get('conveyor.roller');
    expect(derivedValues(roller, defaultParams(roller))).toEqual({ rollerCount: 14 });
    expect(derivedValues(roller, { ...defaultParams(roller), rollerPitch: 0.133 })).toEqual({
      rollerCount: 22,
    });
  });

  it('gives the turntable a swept radius of 972 mm with the defaults (S03)', () => {
    const turntable = createBuiltInRegistry().get('conveyor.chain_turntable');
    expect(derivedValues(turntable, defaultParams(turntable))['sweptRadius']).toBeCloseTo(0.972, 3);
  });
});

describe('validateInstance', () => {
  it('flags a chain conveyor too narrow for a transverse EUR pallet', () => {
    const chain = createBuiltInRegistry().get('conveyor.chain');
    expect(validateInstance(chain, { usableWidth: 1.0 }).map((i) => i.rule)).toEqual([
      'usableWidthFitsPallet',
    ]);
  });

  it('flags a transfer whose transfer chains do not fit between the rollers', () => {
    const transfer = createBuiltInRegistry().get('conveyor.transfer_rc');
    const issues = validateInstance(transfer, {
      rollerPitch: 0.133,
      rollerDiameter: 0.089,
      transferChainWidth: 0.08,
    });
    expect(issues.map((i) => i.rule)).toEqual(['chainFitsBetweenRollers']);
  });

  it('flags a manual lift table whose low height is not below the conveyor height (S12)', () => {
    const table = createBuiltInRegistry().get('conveyor.lift_table_manual');
    expect(
      validateInstance(table, { lowHeight: 0.4, conveyorHeight: 0.4 }).map((i) => i.rule),
    ).toEqual(['heightsReversed']);
  });

  it('flags a scissor table or an elevator that does not change height (S13, S14)', () => {
    const registry = createBuiltInRegistry();
    for (const id of ['conveyor.scissor_lift', 'conveyor.elevator']) {
      const issues = validateInstance(registry.get(id), { entryHeight: 0.6, exitHeight: 0.605 });
      expect(
        issues.map((i) => i.rule),
        id,
      ).toEqual(['noHeightChange']);
    }
  });

  it('flags a lift deck shorter than the pallet plus 100 mm', () => {
    const scissor = createBuiltInRegistry().get('conveyor.scissor_lift');
    expect(validateInstance(scissor, { length: 1.25 }).map((i) => i.rule)).toEqual([
      'deckTooShort',
    ]);
    expect(validateInstance(scissor, { length: 1.3 })).toEqual([]);
  });

  it('checks the rollers under the pallet only on an elevator with a roller deck', () => {
    const elevator = createBuiltInRegistry().get('conveyor.elevator');
    const sparse = { rollerPitch: 0.5 };
    expect(validateInstance(elevator, sparse).map((i) => i.rule)).toEqual([
      'min3RollersUnderPallet',
    ]);
    const chains = {
      ...sparse,
      deckType: 'chains',
      palletSideAlongTravel: 'short',
      usableWidth: 1.3,
    };
    expect(validateInstance(elevator, chains)).toEqual([]);
  });

  it('flags a gravity line whose slope does not beat the rolling resistance (S15)', () => {
    const gravity = createBuiltInRegistry().get('conveyor.gravity');
    expect(
      validateInstance(gravity, { slope: 0.02, rollingResistance: 0.02 }).map((i) => i.rule),
    ).toEqual(['slopeTooLow']);
  });

  it('flags brake rollers so far apart that a pallet can have none under it (S15)', () => {
    const gravity = createBuiltInRegistry().get('conveyor.gravity');
    // 6 × 206.3 mm = 1.238 m > 1.2 m.
    expect(validateInstance(gravity, { brakeEvery: 6 }).map((i) => i.rule)).toEqual([
      'brakeSpacingTooLong',
    ]);
    expect(validateInstance(gravity, { brakeEvery: 5 })).toEqual([]);
  });

  it('flags a pick-up position shorter than the pallet plus 50 mm (S15)', () => {
    const gravity = createBuiltInRegistry().get('conveyor.gravity');
    expect(validateInstance(gravity, { pickupLength: 1.24 }).map((i) => i.rule)).toEqual([
      'pickupTooShort',
    ]);
  });

  it('lets the operator source and sink go down to 50 mm (S12)', () => {
    const registry = createBuiltInRegistry();
    for (const id of ['flow.source', 'flow.sink']) {
      expect(registry.get(id).parameters.find((p) => p.key === 'surfaceHeight')?.min, id).toBe(
        0.05,
      );
    }
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
});

describe('contact materials and pairs', () => {
  it('refers only to known materials, once per unordered pair', () => {
    const materials = new Set(CONTACT_MATERIALS.map((m) => m.id));
    const seen = new Set<string>();
    for (const pair of CONTACT_PAIRS) {
      expect(materials.has(pair.a), pair.a).toBe(true);
      expect(materials.has(pair.b), pair.b).toBe(true);
      const key = [pair.a, pair.b].sort().join('|');
      expect(seen.has(key), key).toBe(false);
      seen.add(key);
    }
  });

  it('keeps each friction default inside its usual range and restitution in [0, 1]', () => {
    for (const pair of CONTACT_PAIRS) {
      expect(pair.mu).toBeGreaterThanOrEqual(pair.range[0]);
      expect(pair.mu).toBeLessThanOrEqual(pair.range[1]);
      expect(pair.restitution).toBeGreaterThanOrEqual(0);
      expect(pair.restitution).toBeLessThanOrEqual(1);
    }
  });

  it('pairs every material with every material, so scenery can touch anything (55 pairs, ADR-0017)', () => {
    for (const a of CONTACT_MATERIALS) {
      for (const b of CONTACT_MATERIALS) {
        expect(() => contactPair(a.id, b.id), `${a.id}–${b.id}`).not.toThrow();
      }
    }
    expect(CONTACT_PAIRS).toHaveLength(55);
  });

  it('offers cardboard for the catalogue boxes (μ = 0.3 on steel rollers, no bounce on rollers)', () => {
    expect(contactPair('cardboard', 'steel-zinc')).toMatchObject({ mu: 0.3, restitution: 0 });
  });

  it('looks pairs up in either order (wood on a steel roller: μ = 0.35)', () => {
    expect(contactPair('wood', 'steel-zinc').mu).toBe(0.35);
    expect(contactPair('steel-zinc', 'wood')).toBe(contactPair('wood', 'steel-zinc'));
  });

  it('grips better on rubber-coated than on steel rollers and bounces more on rubber buffers', () => {
    expect(contactPair('wood', 'rubber').mu).toBeGreaterThan(contactPair('wood', 'steel-zinc').mu);
    expect(contactPair('wood', 'rubber').restitution).toBeGreaterThan(
      contactPair('wood', 'steel-painted').restitution,
    );
  });

  it('rejects a material that is not in the table', () => {
    expect(() => contactPair('rubber', 'glass')).toThrow(/rubber.*glass/);
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

function param(definitionId: string, key: string) {
  return createBuiltInRegistry()
    .get(definitionId)
    .parameters.find((p) => p.key === key);
}
