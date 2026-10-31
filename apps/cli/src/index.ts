// Usage: pnpm run scenario <id | all>
import { type LogEntry, createLogger } from '@intracraft/core-model';
import {
  SCENARIOS,
  type Scenario,
  type ScenarioResult,
  findScenario,
  runScenario,
} from '@intracraft/scenarios';

const logger = createLogger({ write: writeToTerminal });

const requested = process.argv[2] ?? 'all';
const scenarios =
  requested === 'all' ? SCENARIOS : [findScenario(requested)].filter((s) => s !== undefined);

if (scenarios.length === 0) {
  logger.error(
    `Unknown scenario "${requested}". Available: ${SCENARIOS.map((s) => s.id).join(', ')}, all`,
  );
  process.exit(2);
}

const results: ScenarioResult[] = [];
for (const scenario of scenarios) {
  const result = await runScenario(scenario);
  logger.info(formatResult(result, scenario));
  results.push(result);
}

const failed = results.filter((r) => !r.passed);
logger.info(`${results.length - failed.length}/${results.length} scenarios passed`);
process.exit(failed.length === 0 ? 0 : 1);

function formatResult(result: ScenarioResult, scenario: Scenario): string {
  const status = result.passed ? 'PASS' : 'FAIL';
  const unit = scenario.unit;
  const error =
    scenario.absoluteTolerance === undefined
      ? `error ${(result.relativeError * 100).toFixed(3)} %`
      : `error ${result.error.toExponential(2)} ${unit}, limit ±${scenario.absoluteTolerance} ${unit}`;
  return `${status} ${result.id} ${scenario.title.en}: ${result.actual.toFixed(4)} ${unit} (expected ${result.expected} ${unit}, ${error})`;
}

function writeToTerminal(entry: LogEntry): void {
  const stream =
    entry.level === 'error' || entry.level === 'warn' ? process.stderr : process.stdout;
  stream.write(`${entry.message}\n`);
}
