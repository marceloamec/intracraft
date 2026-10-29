// 3D perspective and 2D top (orthographic, looking toward −Y) views of the same layout.
// Components are dropped from the library panel (HTML drag and drop) and moved by dragging;
// both work in either view because the pointer is projected onto the floor plane y = 0.
import type { Vec3 } from '@intracraft/core-model';
import { Canvas, useThree } from '@react-three/fiber';
import { type DragEvent, type PointerEvent, useEffect, useRef } from 'react';
import { Box3, type Camera, type Mesh, type Object3D, Plane, Raycaster, Vector2, Vector3 } from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import { useLayoutStore } from '../layout-store';
import { useEditorStore } from '../store';
import { InstanceModel } from './models/InstanceModel';
import { PortMarkers } from './PortMarkers';

export const DEFINITION_MIME = 'application/x-intracraft-definition';

const TOP_VIEW_ZOOM = 80;
const CLICK_TOLERANCE_PX = 4;
const NORMAL = { ghost: false, selected: false, dimmed: false };

interface Picker {
  floor(clientX: number, clientY: number): Vec3 | null;
  instance(clientX: number, clientY: number): string | null;
}

// Set by <ScenePicker> inside the Canvas so DOM event handlers can raycast with the live camera.
const picker: { current: Picker | null } = { current: null };

export function Viewport() {
  const viewMode = useEditorStore((s) => s.viewMode);
  const isTop = viewMode === 'top';
  const { beginMoveDrag, dragTo, endDrag, select, isDragging } = useLayoutStore.getState();
  const press = useRef<{ x: number; y: number } | null>(null);

  const onPointerDownCapture = (e: PointerEvent<HTMLDivElement>) => {
    press.current = { x: e.clientX, y: e.clientY };
    if (e.button !== 0) return;
    const hit = picker.current?.instance(e.clientX, e.clientY) ?? null;
    const floor = picker.current?.floor(e.clientX, e.clientY) ?? null;
    if (hit === null || floor === null) return;
    // Pressing on an object selects it and starts a move; the camera must not orbit.
    e.stopPropagation();
    select(hit);
    beginMoveDrag(hit, floor);
    const onMove = (ev: globalThis.PointerEvent) => {
      const point = picker.current?.floor(ev.clientX, ev.clientY) ?? null;
      if (point !== null) dragTo(point);
    };
    const onUp = () => {
      window.removeEventListener('pointermove', onMove);
      window.removeEventListener('pointerup', onUp);
      endDrag(true);
    };
    window.addEventListener('pointermove', onMove);
    window.addEventListener('pointerup', onUp);
  };

  const onPointerUp = (e: PointerEvent<HTMLDivElement>) => {
    const start = press.current;
    press.current = null;
    if (start === null || isDragging()) return;
    const isClick = Math.hypot(e.clientX - start.x, e.clientY - start.y) < CLICK_TOLERANCE_PX;
    if (isClick && picker.current?.instance(e.clientX, e.clientY) == null) select(null);
  };

  const onDragOver = (e: DragEvent<HTMLDivElement>) => {
    if (!e.dataTransfer.types.includes(DEFINITION_MIME)) return;
    e.preventDefault();
    e.dataTransfer.dropEffect = 'copy';
    const point = picker.current?.floor(e.clientX, e.clientY) ?? null;
    if (point !== null) dragTo(point);
  };

  const onDrop = (e: DragEvent<HTMLDivElement>) => {
    if (!e.dataTransfer.types.includes(DEFINITION_MIME)) return;
    e.preventDefault();
    endDrag(true);
  };

  return (
    <div
      className="viewport-canvas"
      onPointerDownCapture={onPointerDownCapture}
      onPointerUp={onPointerUp}
      onDragOver={onDragOver}
      onDrop={onDrop}
    >
      <Canvas
        key={viewMode}
        shadows
        orthographic={isTop}
        camera={
          isTop
            ? { position: [2, 20, 0], zoom: TOP_VIEW_ZOOM, up: [0, 0, -1], near: 0.1, far: 100 }
            : { position: [6, 5, 7], fov: 45, near: 0.05, far: 200 }
        }
      >
        <color attach="background" args={['#1e2228']} />
        <ambientLight intensity={0.5} />
        <directionalLight position={[5, 8, 4]} intensity={1.6} castShadow />
        <gridHelper args={[40, 400, '#3a414b', '#2a2f36']} />
        <LayoutScene />
        <ScenePicker />
        <CameraControls target={[2, 0.4, 0]} allowRotate={!isTop} />
      </Canvas>
    </div>
  );
}

function LayoutScene() {
  const layout = useLayoutStore((s) => s.layout);
  const selectedId = useLayoutStore((s) => s.selectedId);
  const preview = useLayoutStore((s) => s.preview);

  return (
    <>
      {layout.instances.map((instance) => (
        <InstanceModel
          key={instance.id}
          instanceId={instance.id}
          definitionId={instance.definitionId}
          params={instance.params}
          position={instance.position}
          yaw={instance.yaw}
          style={{
            ...NORMAL,
            selected: instance.id === selectedId,
            dimmed: instance.id === preview?.instanceId,
          }}
        />
      ))}
      {preview !== null && (
        <InstanceModel
          definitionId={preview.definitionId}
          params={preview.params}
          position={preview.position}
          yaw={preview.yaw}
          style={{ ...NORMAL, ghost: true }}
        />
      )}
      <PortMarkers layout={layout} target={preview?.link?.target ?? null} />
    </>
  );
}

function ScenePicker() {
  const camera = useThree((s) => s.camera);
  const canvas = useThree((s) => s.gl.domElement);
  const scene = useThree((s) => s.scene);

  useEffect(() => {
    const raycaster = new Raycaster();
    const floorPlane = new Plane(new Vector3(0, 1, 0), 0);
    const aim = (clientX: number, clientY: number, cam: Camera) => {
      const rect = canvas.getBoundingClientRect();
      const ndc = new Vector2(((clientX - rect.left) / rect.width) * 2 - 1, -((clientY - rect.top) / rect.height) * 2 + 1);
      raycaster.setFromCamera(ndc, cam);
    };
    picker.current = {
      floor(clientX, clientY) {
        aim(clientX, clientY, camera);
        const hit = raycaster.ray.intersectPlane(floorPlane, new Vector3());
        return hit === null ? null : { x: hit.x, y: 0, z: hit.z };
      },
      instance(clientX, clientY) {
        // Pick by each object's bounding box, so a click between two rollers still hits the conveyor.
        aim(clientX, clientY, camera);
        let best: { id: string; distance: number } | null = null;
        for (const child of scene.children) {
          const id = child.userData['instanceId'];
          if (typeof id !== 'string') continue;
          const hit = raycaster.ray.intersectBox(pickBox(child), new Vector3());
          const distance = hit?.distanceTo(raycaster.ray.origin);
          if (distance !== undefined && (best === null || distance < best.distance)) best = { id, distance };
        }
        return best?.id ?? null;
      },
    };
    return () => {
      picker.current = null;
    };
  }, [camera, canvas, scene]);

  return null;
}

/** Bounding box of an instance's meshes, leaving out floor marks such as the swept circle. */
function pickBox(group: Object3D): Box3 {
  const box = new Box3();
  const part = new Box3();
  group.traverse((o) => {
    if ((o as Mesh).isMesh === true && o.userData['noPick'] !== true) box.union(part.setFromObject(o));
  });
  return box;
}

function CameraControls({ target, allowRotate }: { target: [number, number, number]; allowRotate: boolean }) {
  const camera = useThree((s) => s.camera);
  const domElement = useThree((s) => s.gl.domElement);
  const [tx, ty, tz] = target;

  useEffect(() => {
    const controls = new OrbitControls(camera, domElement);
    controls.target.set(tx, ty, tz);
    controls.enableRotate = allowRotate;
    controls.update();
    return () => controls.dispose();
  }, [camera, domElement, tx, ty, tz, allowRotate]);

  return null;
}
