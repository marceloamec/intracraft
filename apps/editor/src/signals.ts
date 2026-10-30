// Signals the controller rules can use (A08), named `<instance id>.<signal>`: outputs of conveyors
// and sensors for conditions, inputs of conveyors and sources for actions. Mirrors what the 2D
// simulation publishes (sim-des flow-simulation.ts).
import type { Layout } from '@intracraft/core-model';
import type { DefinitionRegistry } from '@intracraft/library';

export interface SignalCatalog {
  /** Signals a condition can read (outputs and inputs). */
  readable: string[];
  /** Signals an action can write (inputs). */
  writable: string[];
}

const CONVEYORS = new Set(['des.roller_conveyor', 'des.chain_conveyor']);

export function signalCatalog(layout: Layout, registry: DefinitionRegistry): SignalCatalog {
  const outputs: string[] = [];
  const inputs: string[] = [];
  for (const instance of layout.instances) {
    const behavior = registry.has(instance.definitionId) ? registry.get(instance.definitionId).behaviors.des : '';
    if (CONVEYORS.has(behavior)) {
      outputs.push(`${instance.id}.running`, `${instance.id}.occupied`);
      inputs.push(`${instance.id}.run`);
    } else if (behavior === 'des.source') {
      inputs.push(`${instance.id}.enable`);
    } else if (behavior === 'des.photoeye') {
      const custom = instance.params['outputSignal'];
      outputs.push(typeof custom === 'string' && custom.trim() !== '' ? custom.trim() : `${instance.id}.detected`);
    }
  }
  return { readable: [...outputs, ...inputs], writable: inputs };
}
