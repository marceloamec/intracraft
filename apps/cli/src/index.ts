// Usage: pnpm run scenario <id | all>
import { type LogEntry, createLogger } from '@intracraft/core-model';
import { SCENARIOS, type ScenarioResult, findScenario, runScenario } from '@intracraft/scenarios';

const logger = createLogger({ write: writeToTerminal });

const requested = process.argv[2] ?? 'all';
const scenarios = requested === 'all' ? SCENARIOS : [findScenario(requested)].filter((s) => s !== undefined);

if (scenarios.length === 0) {
  logger.error(`Unknown scenario "${requested}". Available: ${SCENARIOS.map((s) => s.id).join(', ')}, all`);
  process.exit(2);
}

const results = scenarios.map((scenario) => {
  const result = runScenario(scenario);
  logger.info(formatResult(result, scenario.title.en, scenario.unit));
  return result;
});

const failed = results.filter((r) => !r.passed);
logger.info(`${results.length - failed.length}/${results.length} scenarios passed`);
process.exit(failed.length === 0 ? 0 : 1);

function formatResult(result: ScenarioResult, title: string, unit: string): string {
  const status = result.passed ? 'PASS' : 'FAIL';
  const errorPct = (result.relativeError * 100).toFixed(3);
  return `${status} ${result.id} ${title}: ${result.actual.toFixed(4)} ${unit} (expected ${result.expected} ${unit}, error ${errorPct} %)`;
}

function writeToTerminal(entry: LogEntry): void {
  const stream = entry.level === 'error' || entry.level === 'warn' ? process.stderr : process.stdout;
  stream.write(`${entry.message}\n`);
}
