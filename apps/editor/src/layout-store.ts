// Layout editing state: the command session, the selection and the drag preview ("ghost").
// Every change to the layout goes through `session.execute` (ADR-0004); this store only mirrors
// the resulting layout so React re-renders.
import {
  type CommandInput,
  type CommandName,
  type CommandResult,
  Session,
} from '@intracraft/commands';
import { type Layout, type Vec3, readProjectFile, writeProjectFile } from '@intracraft/core-model';
import {
  type SnapLink,
  type ValidationIssue,
  createBuiltInRegistry,
  resolvePorts,
  instanceParams,
  rotateY,
  snapPose,
} from '@intracraft/library';
import { create } from 'zustand';
import { findMount, isSensor, mountedSensor } from './sensor-mount';

export const registry = createBuiltInRegistry();

const QUARTER_TURN = Math.PI / 2;
export const UNTITLED = 'Untitled';
export const DEFAULT_SEED = 1;

export interface Preview {
  definitionId: string;
  params: Record<string, number | boolean | string>;
  position: Vec3;
  yaw: number;
  link: SnapLink | null;
  issues: ValidationIssue[];
  /** Set when an existing instance is being moved. */
  instanceId?: string;
  /** Sensors: the conveyor it would be mounted on, and the parameters that mounting sets. */
  mount?: { hostId: string; s: number; params: Record<string, string | number> };
}

type Drag =
  | {
      kind: 'new';
      definitionId: string;
      /** Parameters the new object starts with (a catalogue object's model and placement). */
      params: Record<string, number | boolean | string>;
      grab: { x: number; z: number };
      yaw: number;
      moved: boolean;
    }
  | {
      kind: 'move';
      instanceId: string;
      grab: { x: number; z: number };
      yaw: number;
      moved: boolean;
      start: Vec3;
    };

export interface Status {
  message: string;
  issues: ValidationIssue[];
  /** What the message reports: a snap link (default), a saved file or an opened file. */
  kind?: 'saved' | 'opened';
}

interface LayoutState {
  layout: Layout;
  canUndo: boolean;
  canRedo: boolean;
  selectedId: string | null;
  preview: Preview | null;
  status: Status | null;
  /** Saved with the layout in the project file (A04). */
  projectName: string;
  seed: number;
  setProjectInfo(info: { projectName?: string; seed?: number }): void;
  setStatus(status: Status): void;
  /** Loads a project file's text as one undoable command; returns an error text when it fails. */
  openProject(text: string): string | null;
  /** Text of the project file for the current layout. */
  projectText(): string;
  run<N extends CommandName>(name: N, input: CommandInput<N>, message?: string): CommandResult;
  undo(): void;
  redo(): void;
  select(id: string | null): void;
  rotateSelected(): void;
  deleteSelected(): void;
  beginNewDrag(definitionId: string, params?: Record<string, number | boolean | string>): void;
  beginMoveDrag(instanceId: string, pointer: Vec3): void;
  dragTo(pointer: Vec3): void;
  rotateDrag(): void;
  endDrag(commit: boolean): void;
  isDragging(): boolean;
}

const session = new Session(registry);
let drag: Drag | null = null;
let lastPointer: Vec3 | null = null;
const MOVE_THRESHOLD = 0.02; // m of pointer travel before a press becomes a move

export const useLayoutStore = create<LayoutState>()((set, get) => {
  const sync = (status: Status | null) =>
    set({ layout: session.layout, canUndo: session.canUndo, canRedo: session.canRedo, status });

  return {
    layout: session.layout,
    canUndo: false,
    canRedo: false,
    selectedId: null,
    preview: null,
    status: null,
    projectName: UNTITLED,
    seed: DEFAULT_SEED,

    setProjectInfo: (info) => set(info),
    setStatus: (status) => set({ status }),
    openProject(text) {
      const file = readProjectFile(text);
      if (!file.ok) return file.error;
      const result = get().run('layout.load', { layout: file.project.layout });
      if (!result.ok) return result.issues.map((i) => i.message).join(' · ');
      set({ projectName: file.project.name, seed: file.project.seed, selectedId: null });
      return null;
    },
    projectText: () => writeProjectFile(get().projectName, get().seed, session.layout),

    run(name, input, message) {
      const result = session.execute(name, input, 'user');
      sync({ message: result.ok ? (message ?? '') : '', issues: result.issues });
      return result;
    },
    undo() {
      session.undo('user');
      sync(null);
      if (!session.layout.instances.some((i) => i.id === get().selectedId))
        set({ selectedId: null });
    },
    redo() {
      session.redo('user');
      sync(null);
      if (!session.layout.instances.some((i) => i.id === get().selectedId))
        set({ selectedId: null });
    },
    select: (selectedId) => set({ selectedId }),
    rotateSelected() {
      const instance = session.layout.instances.find((i) => i.id === get().selectedId);
      if (instance === undefined) return;
      // Turn about the object's centre, not its origin, so it stays under the cursor.
      const yaw = instance.yaw + QUARTER_TURN;
      const centre = worldCentre(
        instance.definitionId,
        instance.params,
        instance.position,
        instance.yaw,
      );
      const offset = rotateY(localCentre(instance.definitionId, instance.params), yaw);
      const position = { x: centre.x - offset.x, y: instance.position.y, z: centre.z - offset.z };
      get().run('layout.move', { instanceId: instance.id, position, yaw });
    },
    deleteSelected() {
      const id = get().selectedId;
      if (id === null) return;
      if (get().run('layout.remove', { instanceId: id }).ok) set({ selectedId: null });
    },

    beginNewDrag(definitionId, params = {}) {
      drag = { kind: 'new', definitionId, params, grab: { x: 0, z: 0 }, yaw: 0, moved: true };
    },
    beginMoveDrag(instanceId, pointer) {
      const instance = session.layout.instances.find((i) => i.id === instanceId);
      if (instance === undefined) return;
      const centre = worldCentre(
        instance.definitionId,
        instance.params,
        instance.position,
        instance.yaw,
      );
      drag = {
        kind: 'move',
        instanceId,
        grab: { x: pointer.x - centre.x, z: pointer.z - centre.z },
        yaw: instance.yaw,
        moved: false,
        start: pointer,
      };
      lastPointer = pointer;
    },
    dragTo(pointer) {
      if (drag === null) return;
      lastPointer = pointer;
      if (drag.kind === 'move' && !drag.moved) {
        if (Math.hypot(pointer.x - drag.start.x, pointer.z - drag.start.z) < MOVE_THRESHOLD) return;
        drag.moved = true;
      }
      set({ preview: computePreview(drag, pointer) });
    },
    rotateDrag() {
      if (drag === null) return;
      drag.yaw += QUARTER_TURN;
      if (lastPointer !== null && drag.moved) set({ preview: computePreview(drag, lastPointer) });
    },
    endDrag(commit) {
      const { preview } = get();
      const finished = drag;
      drag = null;
      lastPointer = null;
      set({ preview: null });
      if (!commit || preview === null || finished === null || !finished.moved) return;
      const status =
        preview.mount !== undefined
          ? mountText(preview.mount)
          : preview.link === null
            ? ''
            : linkText(preview.link);
      const params =
        preview.mount?.params ??
        (finished.kind === 'new' && Object.keys(preview.params).length > 0
          ? preview.params
          : undefined);
      if (finished.kind === 'new') {
        const result = get().run(
          'layout.addInstance',
          {
            definitionId: preview.definitionId,
            position: preview.position,
            yaw: preview.yaw,
            ...(params && { params }),
          },
          status,
        );
        if (result.instanceId !== undefined) set({ selectedId: result.instanceId });
      } else {
        get().run(
          'layout.move',
          {
            instanceId: finished.instanceId,
            position: preview.position,
            yaw: preview.yaw,
            ...(params && { params }),
          },
          status,
        );
      }
    },
    isDragging: () => drag !== null,
  };
});

function computePreview(current: Drag, pointer: Vec3): Preview {
  const instance =
    current.kind === 'move'
      ? session.layout.instances.find((i) => i.id === current.instanceId)
      : undefined;
  const definitionId =
    instance?.definitionId ?? (current.kind === 'new' ? current.definitionId : '');
  const params = instance?.params ?? (current.kind === 'new' ? current.params : {});
  if (isSensor(definitionId, registry))
    return sensorPreview(definitionId, params, pointer, instance?.id);
  const offset = rotateY(localCentre(definitionId, params), current.yaw);
  // Scenery objects stand on whatever is under the pointer (S17); everything else on the floor.
  const onSupport = registry.get(definitionId).category === 'scenery';
  const position = {
    x: pointer.x - current.grab.x - offset.x,
    y: onSupport ? pointer.y : 0,
    z: pointer.z - current.grab.z - offset.z,
  };
  const candidate = { definition: registry.get(definitionId), params, position, yaw: current.yaw };
  const snapped = snapPose(
    instance === undefined ? candidate : { ...candidate, instanceId: instance.id },
    session.layout,
    registry,
  );
  const preview: Preview = { definitionId, params, ...snapped };
  if (instance !== undefined) preview.instanceId = instance.id;
  return preview;
}

/** Centre of the footprint in the local frame, taken as the mean of the port positions. */
function localCentre(
  definitionId: string,
  params: Record<string, number | boolean | string>,
): { x: number; z: number } {
  const definition = registry.get(definitionId);
  const ports = resolvePorts(definition, instanceParams(definition, params));
  if (ports.length === 0) return { x: 0, z: 0 };
  const sum = ports.reduce((s, p) => ({ x: s.x + p.local.x, z: s.z + p.local.z }), { x: 0, z: 0 });
  return { x: sum.x / ports.length, z: sum.z / ports.length };
}

function worldCentre(
  definitionId: string,
  params: Record<string, number | boolean | string>,
  position: Vec3,
  yaw: number,
): { x: number; z: number } {
  const offset = rotateY(localCentre(definitionId, params), yaw);
  return { x: position.x + offset.x, z: position.z + offset.z };
}

/** A sensor follows the conveyor under the pointer; off a conveyor it is placed free (and detects nothing). */
function sensorPreview(
  definitionId: string,
  params: Record<string, number | boolean | string>,
  pointer: Vec3,
  instanceId: string | undefined,
): Preview {
  const mount = findMount(session.layout, registry, pointer);
  const preview: Preview =
    mount === undefined
      ? {
          definitionId,
          params: { ...params, host: '' },
          position: { x: pointer.x, y: 0, z: pointer.z },
          yaw: 0,
          link: null,
          issues: [
            {
              rule: 'sensorWithoutHost',
              severity: 'warning',
              message: 'Drop the sensor on a roller or chain conveyor to mount it.',
            },
          ],
          mount: { hostId: '', s: 0, params: { host: '' } },
        }
      : (() => {
          const mounted = mountedSensor(mount);
          return {
            definitionId,
            params: { ...params, ...mounted.params },
            position: mounted.position,
            yaw: mounted.yaw,
            link: null,
            issues: [],
            mount: { hostId: mount.host.id, s: mount.s, params: mounted.params },
          };
        })();
  if (instanceId !== undefined) preview.instanceId = instanceId;
  return preview;
}

function mountText(mount: NonNullable<Preview['mount']>): string {
  return mount.hostId === '' ? '' : `${mount.hostId} · ${Math.round(mount.s * 1000)} mm`;
}

function linkText(link: SnapLink): string {
  return `${link.movingPort} → ${link.target.instanceId}.${link.target.portId}`;
}
