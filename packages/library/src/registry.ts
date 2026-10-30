// Built-in definitions, validated at load time. User and project libraries will register here too.
import chainConveyorJson from './definitions/chain-conveyor.json';
import chainTurntableJson from './definitions/chain-turntable.json';
import elevatorJson from './definitions/elevator.json';
import gravityLineJson from './definitions/gravity-line.json';
import liftTableManualJson from './definitions/lift-table-manual.json';
import photoeyeJson from './definitions/photoeye.json';
import processStationJson from './definitions/process-station.json';
import rollerConveyorJson from './definitions/roller-conveyor.json';
import rollerTurntableJson from './definitions/roller-turntable.json';
import scissorLiftJson from './definitions/scissor-lift.json';
import sinkJson from './definitions/sink.json';
import sourceJson from './definitions/source.json';
import transferRcJson from './definitions/transfer-rc.json';
import { type ComponentDefinition, componentDefinitionSchema } from './definition-schema';

const BUILT_IN_SOURCES: unknown[] = [
  rollerConveyorJson,
  chainConveyorJson,
  chainTurntableJson,
  rollerTurntableJson,
  transferRcJson,
  liftTableManualJson,
  scissorLiftJson,
  elevatorJson,
  gravityLineJson,
  sourceJson,
  sinkJson,
  processStationJson,
  photoeyeJson,
];

export class DefinitionRegistry {
  private readonly definitions = new Map<string, ComponentDefinition>();

  register(source: unknown): ComponentDefinition {
    const definition = componentDefinitionSchema.parse(source);
    if (this.definitions.has(definition.id)) {
      throw new Error(`Definition "${definition.id}" is already registered`);
    }
    this.definitions.set(definition.id, definition);
    return definition;
  }

  has(id: string): boolean {
    return this.definitions.has(id);
  }

  get(id: string): ComponentDefinition {
    const definition = this.definitions.get(id);
    if (definition === undefined) throw new Error(`Unknown definition "${id}"`);
    return definition;
  }

  list(): ComponentDefinition[] {
    return [...this.definitions.values()];
  }
}

export function createBuiltInRegistry(): DefinitionRegistry {
  const registry = new DefinitionRegistry();
  for (const source of BUILT_IN_SOURCES) registry.register(source);
  return registry;
}

/** Default parameter values of a definition, keyed by parameter key (derived values excluded). */
export function defaultParams(definition: ComponentDefinition): Record<string, number | boolean | string> {
  const params: Record<string, number | boolean | string> = {};
  for (const p of definition.parameters) {
    if (!p.derived && p.default !== undefined) params[p.key] = p.default;
  }
  return params;
}
