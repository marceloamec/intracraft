// Layout scenarios: a saved project file run in the 2D mode for a fixed time; one statistic of
// one object is the result. The files in scenarios/layouts/ also serve as examples to open.
import { readProjectFile, secondsToSimTime } from '@intracraft/core-model';
import { createBuiltInRegistry, deriveConnections } from '@intracraft/library';
import { FlowSimulation, buildFlowNetwork } from '@intracraft/sim-des';
import { z } from 'zod';
import demoElevatorC from '../scenarios/layouts/demo-elevator-c.intracraft.json';
import demoManualOutput from '../scenarios/layouts/demo-manual-output.intracraft.json';
import demoProcessLine from '../scenarios/layouts/demo-process-line.intracraft.json';
import demoRollerLine from '../scenarios/layouts/demo-roller-line.intracraft.json';
import demoTurntableSplit from '../scenarios/layouts/demo-turntable-split.intracraft.json';
import flowT1 from '../scenarios/layouts/flow-t1.intracraft.json';
import flowT2 from '../scenarios/layouts/flow-t2.intracraft.json';
import referenceAllFive from '../scenarios/layouts/reference-all-five.intracraft.json';

/** Project files by name (file name without `.intracraft.json`). */
export const LAYOUT_FILES: Record<string, unknown> = {
  'demo-elevator-c': demoElevatorC,
  'demo-manual-output': demoManualOutput,
  'demo-process-line': demoProcessLine,
  'demo-roller-line': demoRollerLine,
  'demo-turntable-split': demoTurntableSplit,
  'flow-t1': flowT1,
  'flow-t2': flowT2,
  'reference-all-five': referenceAllFive,
};

export const flowInputsSchema = z.object({
  layout: z.string(),
  seconds: z.number().positive(),
  instanceId: z.string(),
  statistic: z.string(),
});

export type FlowInputs = z.infer<typeof flowInputsSchema>;

const registry = createBuiltInRegistry();

/** Runs a layout file in the 2D mode and returns the requested statistic. */
export function runFlowScenario({ layout, seconds, instanceId, statistic }: FlowInputs): number {
  const file = readProjectFile(JSON.stringify(LAYOUT_FILES[layout] ?? null));
  if (!file.ok) throw new Error(`Layout "${layout}": ${file.error}`);
  const project = file.project;
  // Files store no connections: they are derived from the geometry, as in the editor.
  const connected = { ...project.layout, connections: deriveConnections(project.layout, registry) };
  const sim = new FlowSimulation(buildFlowNetwork(connected, registry), { seed: project.seed });
  sim.runUntil(secondsToSimTime(seconds));
  const value = sim.statistics()[instanceId]?.[statistic];
  if (value === undefined)
    throw new Error(`Layout "${layout}" has no statistic ${instanceId}.${statistic}`);
  return value;
}
