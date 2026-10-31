// Reference scenarios: analytic T1–T5 (MVP plan §15.2), the same checks run on saved layouts in the
// 2D mode, and the physics bench runs P1–P7. Each one states its inputs in SI units and the expected
// result with a relative or an absolute tolerance.
import { localizedTextSchema } from '@intracraft/core-model';
import { impactEnergy, maxThroughput, profileTime, transferCycleTime } from '@intracraft/sim-des';
import { z } from 'zod';
import scenariosJson from '../scenarios/analytic.json';
import layoutScenariosJson from '../scenarios/layouts.json';
import physicsScenariosJson from '../scenarios/physics.json';
import { flowInputsSchema, runFlowScenario } from './flow';
import { physicsInputsSchema, runPhysicsScenario } from './physics';

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
  z.object({ kind: z.literal('flow'), inputs: flowInputsSchema }),
  z.object({ kind: z.literal('physics'), inputs: physicsInputsSchema }),
]);

export const scenarioSchema = z
  .intersection(
    z.object({
      id: z.string(),
      title: localizedTextSchema,
      unit: z.string(),
      expected: z.number(),
      /** |actual − expected| / |expected| must stay below it. */
      relativeTolerance: z.number().positive().optional(),
      /** |actual − expected| must not exceed it (for expected values of 0, e.g. a position error). */
      absoluteTolerance: z.number().nonnegative().optional(),
    }),
    analyticScenarioSchema,
  )
  .refine((s) => (s.relativeTolerance === undefined) !== (s.absoluteTolerance === undefined), {
    message: 'A scenario needs exactly one of relativeTolerance and absoluteTolerance',
  });

export type Scenario = z.infer<typeof scenarioSchema>;

export interface ScenarioResult {
  id: string;
  actual: number;
  expected: number;
  /** actual − expected, in the scenario unit. */
  error: number;
  relativeError: number;
  passed: boolean;
}

export const SCENARIOS: Scenario[] = z
  .array(scenarioSchema)
  .parse([...scenariosJson, ...layoutScenariosJson, ...physicsScenariosJson]);

export function findScenario(id: string): Scenario | undefined {
  return SCENARIOS.find((s) => s.id.toLowerCase() === id.toLowerCase());
}

export async function runScenario(scenario: Scenario): Promise<ScenarioResult> {
  const actual = await evaluate(scenario);
  const error = actual - scenario.expected;
  const relativeError = Math.abs(error) / Math.abs(scenario.expected);
  return {
    id: scenario.id,
    actual,
    expected: scenario.expected,
    error,
    relativeError,
    passed:
      scenario.absoluteTolerance === undefined
        ? relativeError < (scenario.relativeTolerance ?? 0)
        : Math.abs(error) <= scenario.absoluteTolerance,
  };
}

async function evaluate(scenario: Scenario): Promise<number> {
  switch (scenario.kind) {
    case 'profileTime': {
      const { decel, ...rest } = scenario.inputs;
      return profileTime(decel === undefined ? rest : { ...rest, decel });
    }
    case 'throughput':
      return maxThroughput(
        scenario.inputs.speed,
        scenario.inputs.palletLength,
        scenario.inputs.gap,
      );
    case 'transferCycle':
      return transferCycleTime(scenario.inputs);
    case 'impactEnergy':
      return impactEnergy(scenario.inputs.mass, scenario.inputs.speed);
    case 'flow':
      return runFlowScenario(scenario.inputs);
    case 'physics':
      return runPhysicsScenario(scenario.inputs);
  }
}
