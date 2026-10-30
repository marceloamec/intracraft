// One editing session: the current layout, the command catalogue, undo/redo and the command log.
// The editor, the CLI and (later) MCP and chat all change the layout only through `execute`.
import type { Layout } from '@intracraft/core-model';
import { type DefinitionRegistry, type ValidationIssue, deriveConnections } from '@intracraft/library';
import type { z } from 'zod';
import { addInstance } from './add-instance';
import { clearLayout } from './clear-layout';
import { loadLayout } from './load-layout';
import type { Command } from './command';
import { moveInstance } from './move-instance';
import { removeInstance } from './remove-instance';
import { setAttributes } from './set-attributes';
import { setParams } from './set-params';
import { setRules } from './set-rules';

export const COMMANDS = {
  'layout.addInstance': addInstance,
  'layout.move': moveInstance,
  'layout.setParams': setParams,
  'layout.setAttributes': setAttributes,
  'layout.remove': removeInstance,
  'layout.clear': clearLayout,
  'layout.load': loadLayout,
  'control.setRules': setRules,
} as const;

export type CommandName = keyof typeof COMMANDS;
export type CommandInput<N extends CommandName> = z.input<(typeof COMMANDS)[N]['input']>;

/** Who triggered a change (A15 §1). */
export type Actor = 'user' | 'cli' | 'chat' | `mcp:${string}`;

export interface CommandResult {
  ok: boolean;
  /** Rejection reasons when not ok; validation warnings of the new state when ok. */
  issues: ValidationIssue[];
  instanceId?: string;
}

export interface LogEntry {
  seq: number;
  actor: Actor;
  command: CommandName | 'edit.undo' | 'edit.redo';
  input: unknown;
  ok: boolean;
  issues: ValidationIssue[];
}

const EMPTY_LAYOUT: Layout = { instances: [], connections: [] };

export class Session {
  readonly log: LogEntry[] = [];
  private current: Layout;
  private readonly undoStack: Layout[] = [];
  private readonly redoStack: Layout[] = [];

  constructor(
    readonly registry: DefinitionRegistry,
    layout: Layout = EMPTY_LAYOUT,
  ) {
    this.current = layout;
  }

  get layout(): Layout {
    return this.current;
  }

  get canUndo(): boolean {
    return this.undoStack.length > 0;
  }

  get canRedo(): boolean {
    return this.redoStack.length > 0;
  }

  /** Validates and applies one command atomically: either the whole change happens or nothing does. */
  execute<N extends CommandName>(name: N, input: CommandInput<N>, actor: Actor): CommandResult {
    const command = COMMANDS[name] as Command<z.ZodType>;
    const parsed = command.input.safeParse(input);
    const outcome = parsed.success
      ? command.apply(this.current, parsed.data, { registry: this.registry })
      : { ok: false as const, issues: [{ rule: 'invalidInput', severity: 'error' as const, message: parsed.error.message }] };

    if (outcome.ok) {
      this.undoStack.push(this.current);
      this.redoStack.length = 0;
      this.current = { ...outcome.layout, connections: deriveConnections(outcome.layout, this.registry) };
    }
    this.record(actor, name, input, outcome.ok, outcome.issues);
    const result: CommandResult = { ok: outcome.ok, issues: outcome.issues };
    if (outcome.ok && outcome.instanceId !== undefined) result.instanceId = outcome.instanceId;
    return result;
  }

  undo(actor: Actor): boolean {
    return this.step(this.undoStack, this.redoStack, actor, 'edit.undo');
  }

  redo(actor: Actor): boolean {
    return this.step(this.redoStack, this.undoStack, actor, 'edit.redo');
  }

  /** Re-runs a logged entry, e.g. to rebuild a session from its command log. */
  replay(entry: LogEntry): void {
    if (entry.command === 'edit.undo') this.undo(entry.actor);
    else if (entry.command === 'edit.redo') this.redo(entry.actor);
    else this.execute(entry.command, entry.input as CommandInput<CommandName>, entry.actor);
  }

  private step(from: Layout[], to: Layout[], actor: Actor, command: 'edit.undo' | 'edit.redo'): boolean {
    const layout = from.pop();
    if (layout !== undefined) {
      to.push(this.current);
      this.current = layout;
    }
    this.record(actor, command, {}, layout !== undefined, []);
    return layout !== undefined;
  }

  private record(actor: Actor, command: LogEntry['command'], input: unknown, ok: boolean, issues: ValidationIssue[]): void {
    this.log.push({ seq: this.log.length + 1, actor, command, input, ok, issues });
  }
}
