// Analytic reference scenarios T1–T5 (MVP plan §15.2). Each one states its inputs in SI units
// and the expected result; running it compares the 2D-mode formula against that expectation.
import { localizedTextSchema } from '@intracraft/core-model';
import {
  impactEnergy,
  maxThroughput,
  profileTime,
  transferCycleTime,
} from '@intracraft/sim-des';
import { z } from 'zod';
import scenariosJson from '../scenarios/analytic.json';

export const analyticScenarioSchema = z.discriminatedUnion('kind', [
  z.object({
    kind: z.literal('profileTime'),
    inputs: z.object({
      distance: z.number(),
      maxSpeed: z.number(),
      accel: z.number(),
      decel: z.number().optional(),
    }),
  }),
  z.object({
    kind: z.literal('throughput'),
    inputs: z.object({ speed: z.number(), palletLength: z.number(), gap: z.number() }),
  }),
  z.object({
    kind: z.literal('transferCycle'),
    inputs: z.object({
      liftTime: z.number(),
      lowerTime: z.number(),
      distance: z.number(),
      speed: z.number(),
      accel: z.number(),
    }),
  }),
  z.object({
    kind: z.literal('impactEnergy'),
    inputs: z.object({ mass: z.number(), speed: z.number() }),
  }),
]);

export const scenarioSchema = z.intersection(
  z.object({
    id: z.string(),
    title: localizedTextSchema,
    unit: z.string(),
    expected: z.number(),
    relativeTolerance: z.number().positive(),
  }),
  analyticScenarioSchema,
);

export type Scenario = z.infer<typeof scenarioSchema>;

export interface ScenarioResult {
  id: string;
  actual: number;
  expected: number;
  relativeError: number;
  passed: boolean;
}

export const SCENARIOS: Scenario[] = z.array(scenarioSchema).parse(scenariosJson);

export function findScenario(id: string): Scenario | undefined {
  return SCENARIOS.find((s) => s.id.toLowerCase() === id.toLowerCase());
}

export function runScenario(scenario: Scenario): ScenarioResult {
  const actual = evaluate(scenario);
  const relativeError = Math.abs(actual - scenario.expected) / Math.abs(scenario.expected);
  return {
    id: scenario.id,
    actual,
    expected: scenario.expected,
    relativeError,
    passed: relativeError < scenario.relativeTolerance,
  };
}

function evaluate(scenario: Scenario): number {
  switch (scenario.kind) {
    case 'profileTime': {
      const { decel, ...rest } = scenario.inputs;
      return profileTime(decel === undefined ? rest : { ...rest, decel });
    }
    case 'throughput':
      return maxThroughput(scenario.inputs.speed, scenario.inputs.palletLength, scenario.inputs.gap);
    case 'transferCycle':
      return transferCycleTime(scenario.inputs);
    case 'impactEnergy':
      return impactEnergy(scenario.inputs.mass, scenario.inputs.speed);
  }
}
