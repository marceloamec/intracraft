// Shared types of the command layer. A command validates its input with Zod and turns one layout
// into the next without mutating it; the session adds connections, undo and the log.
import type { Instance, Layout } from '@intracraft/core-model';
import type { DefinitionRegistry, ValidationIssue } from '@intracraft/library';
import type { z } from 'zod';

export interface CommandContext {
  registry: DefinitionRegistry;
}

export type Outcome =
  | { ok: true; layout: Layout; instanceId?: string; issues: ValidationIssue[] }
  | { ok: false; issues: ValidationIssue[] };

export interface Command<S extends z.ZodType> {
  name: string;
  input: S;
  apply(layout: Layout, input: z.output<S>, context: CommandContext): Outcome;
}

export function defineCommand<S extends z.ZodType>(command: Command<S>): Command<S> {
  return command;
}

export function fail(rule: string, message: string): Outcome {
  return { ok: false, issues: [{ rule, severity: 'error', message }] };
}

/** Finds an instance or returns the rejection outcome. */
export function findInstance(layout: Layout, instanceId: string): Instance | Outcome {
  return layout.instances.find((i) => i.id === instanceId) ?? fail('unknownInstance', `No instance "${instanceId}"`);
}

export function isOutcome(value: Instance | Outcome): value is Outcome {
  return 'ok' in value;
}

/** Layout with one instance replaced. */
export function replaceInstance(layout: Layout, instance: Instance): Layout {
  return { ...layout, instances: layout.instances.map((i) => (i.id === instance.id ? instance : i)) };
}
