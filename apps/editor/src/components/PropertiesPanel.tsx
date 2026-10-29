// Properties of the selected instance, generated from its definition (AGENTS.md: no hand-written
// forms). Every edit is a command: layout.setAttributes, layout.move or layout.setParams.
import type { Instance } from '@intracraft/core-model';
import { localize, radToDeg, degToRad } from '@intracraft/core-model';
import {
  type ComponentDefinition,
  type ParameterDefinition,
  connectionIssues,
  derivedValues,
  instanceParams,
  validateInstance,
  worldPorts,
} from '@intracraft/library';
import { useT } from '../i18n/useT';
import { registry, useLayoutStore } from '../layout-store';
import { useEditorStore } from '../store';
import { AttributesEditor } from './AttributesEditor';
import { NumberInput, ParamField } from './ParamField';

export function PropertiesPanel() {
  const t = useT();
  const layout = useLayoutStore((s) => s.layout);
  const selectedId = useLayoutStore((s) => s.selectedId);
  const instance = layout.instances.find((i) => i.id === selectedId);

  return (
    <aside className="side-panel properties-panel" aria-label={t('propertiesTitle')}>
      <h2>{t('propertiesTitle')}</h2>
      {instance === undefined ? (
        <p className="hint">
          {t('nothingSelected')}
          <br />
          {t('layoutSummary')
            .replace('{objects}', String(layout.instances.length))
            .replace('{connections}', String(layout.connections.length))}
        </p>
      ) : (
        <InstanceProperties key={instance.id} instance={instance} definition={registry.get(instance.definitionId)} />
      )}
    </aside>
  );
}

function InstanceProperties({ instance, definition }: { instance: Instance; definition: ComponentDefinition }) {
  const t = useT();
  const language = useEditorStore((s) => s.language);
  const layout = useLayoutStore((s) => s.layout);
  const run = useLayoutStore((s) => s.run);
  const values = instanceParams(definition, instance.params);
  const derived = derivedValues(definition, values);
  const issues = [...validateInstance(definition, instance.params), ...connectionIssues(layout, registry, instance.id)];
  const move = (position: Instance['position'], yaw: number) =>
    run('layout.move', { instanceId: instance.id, position, yaw }).ok;

  return (
    <>
      <section className="prop-section">
        <h3>{t('identity')}</h3>
        <label className="prop-row">
          <span>{t('name')}</span>
          <input
            key={instance.name}
            defaultValue={instance.name}
            onBlur={(e) => {
              const name = e.currentTarget.value.trim();
              if (name === '' || name === instance.name) e.currentTarget.value = instance.name;
              else run('layout.setAttributes', { instanceId: instance.id, name });
            }}
            onKeyDown={blurOnEnter}
          />
        </label>
        <div className="prop-row">
          <span>{t('id')}</span>
          <code>{instance.id}</code>
        </div>
        <div className="prop-row">
          <span>{t('type')}</span>
          <span>{localize(definition.name, language)}</span>
        </div>
        <NumberInput
          label={t('positionX')}
          unit="mm"
          value={instance.position.x * 1000}
          onCommit={(x) => move({ ...instance.position, x: x / 1000 }, instance.yaw)}
        />
        <NumberInput
          label={t('positionZ')}
          unit="mm"
          value={instance.position.z * 1000}
          onCommit={(z) => move({ ...instance.position, z: z / 1000 }, instance.yaw)}
        />
        <NumberInput
          label={t('rotation')}
          unit="°"
          value={radToDeg(instance.yaw)}
          onCommit={(deg) => move(instance.position, degToRad(deg))}
        />
      </section>

      <section className="prop-section">
        <h3>{t('attributes')}</h3>
        <AttributesEditor
          attributes={instance.attributes}
          onChange={(attributes) => run('layout.setAttributes', { instanceId: instance.id, attributes }).ok}
        />
      </section>

      {groupParameters(definition.parameters).map(([group, parameters]) => {
        const basic = parameters.filter((p) => p.advanced !== true);
        const advanced = parameters.filter((p) => p.advanced === true);
        const field = (p: ParameterDefinition) => (
          <ParamField
            key={p.key}
            parameter={p}
            value={p.derived === true ? derived[p.key] : values[p.key]}
            disabled={!dependencyMet(p, values)}
            onCommit={(value) => run('layout.setParams', { instanceId: instance.id, params: { [p.key]: value } }).ok}
          />
        );
        return (
          <section key={group} className="prop-section">
            <h3>{groupLabel(group)}</h3>
            {basic.map(field)}
            {advanced.length > 0 && (
              <details>
                <summary>{t('advanced')}</summary>
                {advanced.map(field)}
              </details>
            )}
          </section>
        );
      })}

      <section className="prop-section">
        <h3>{t('ports')}</h3>
        <ul className="port-list">
          {worldPorts(instance, definition).map((port) => {
            const link = layout.connections.find(
              (c) =>
                (c.from.instanceId === instance.id && c.from.portId === port.portId) ||
                (c.to.instanceId === instance.id && c.to.portId === port.portId),
            );
            const other = link === undefined ? null : link.from.instanceId === instance.id ? link.to : link.from;
            return (
              <li key={port.portId}>
                <span className={other === null ? 'port-dot free' : 'port-dot connected'} />
                <strong>{port.portId}</strong> <small>({port.direction})</small>{' '}
                {other === null ? t('portFree') : `→ ${other.instanceId}.${other.portId}`}
              </li>
            );
          })}
        </ul>
      </section>

      {issues.length > 0 && (
        <section className="prop-section">
          <h3>{t('warnings')}</h3>
          <ul className="issue-list">
            {issues.map((issue, i) => (
              <li key={i} className={issue.severity}>
                {issue.message}
              </li>
            ))}
          </ul>
        </section>
      )}
    </>
  );
}

function groupParameters(parameters: ParameterDefinition[]): [string, ParameterDefinition[]][] {
  const groups = new Map<string, ParameterDefinition[]>();
  for (const p of parameters) groups.set(p.group, [...(groups.get(p.group) ?? []), p]);
  return [...groups];
}

/** `dependsOn` has the form "key=value": the field is only active when that holds. */
function dependencyMet(p: ParameterDefinition, values: Record<string, number | boolean | string>): boolean {
  if (p.dependsOn === undefined) return true;
  const [key, expected] = p.dependsOn.split('=');
  return key === undefined || expected === undefined || String(values[key]) === expected;
}

function groupLabel(group: string): string {
  return group.charAt(0).toUpperCase() + group.slice(1);
}

function blurOnEnter(e: React.KeyboardEvent<HTMLInputElement>) {
  if (e.key === 'Enter') e.currentTarget.blur();
}
