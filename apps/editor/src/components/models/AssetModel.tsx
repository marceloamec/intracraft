// A scenery object (S17): its catalogue model standing on its support (a dropped object is shown at
// its start height). While a 3D run reports a pose for it, the model follows that pose. Until the
// file is loaded, or when it was never downloaded, an outline of the model's bounding box is drawn.
import { builtInCatalog } from '@intracraft/asset-catalog';
import { useFrame } from '@react-three/fiber';
import { useContext, useMemo, useRef } from 'react';
import {
  type Group,
  type Material,
  Matrix4,
  type Mesh,
  type Object3D,
  Quaternion,
  Vector3,
} from 'three';
import { useAssetModel } from '../../asset-models';
import { liveFrame } from '../../sim-store';
import { type ModelStyle, ModelStyleContext, n } from './parts';
import type { ModelProps } from './types';

const SELECTED_EMISSIVE = '#2a6fd6';

export function AssetModel({ params, instanceId }: ModelProps) {
  const style = useContext(ModelStyleContext);
  const assetId = String(params['asset']);
  const asset = builtInCatalog.list().find((a) => a.id === assetId);
  const state = useAssetModel(assetId);
  const inner = useRef<Group>(null);
  const gap = params['placement'] === 'drop' ? n(params, 'dropHeight') : 0;
  const lift = (asset === undefined ? 0 : -asset.bounds.min.y) + gap;
  // A styled copy keeps the plain one untouched, so the style can change back.
  const styled = useMemo(
    () => (state.status === 'ready' ? withStyle(state.scene, style) : null),
    [state, style],
  );
  const scratch = useMemo(() => ({ matrix: new Matrix4(), parent: new Matrix4() }), []);

  useFrame(() => {
    const group = inner.current;
    if (group === null) return;
    const view = instanceId === undefined ? undefined : liveFrame.snapshot?.objects?.[instanceId];
    if (view === undefined) {
      group.position.set(0, lift, 0);
      group.quaternion.identity();
      return;
    }
    // The physics pose is in world coordinates; the model sits inside the instance's group.
    const { position: p, rotation: r } = view.pose;
    group.parent?.updateWorldMatrix(true, false);
    scratch.parent.copy(group.parent?.matrixWorld ?? scratch.parent.identity()).invert();
    scratch.matrix.compose(
      new Vector3(p.x, p.y, p.z),
      new Quaternion(r.x, r.y, r.z, r.w),
      new Vector3(1, 1, 1),
    );
    scratch.matrix
      .premultiply(scratch.parent)
      .decompose(group.position, group.quaternion, group.scale);
  });

  if (asset === undefined) return <></>;
  const { min, max } = asset.bounds;
  return (
    <group ref={inner} position={[0, lift, 0]}>
      {styled !== null ? (
        <primitive object={styled} />
      ) : (
        <mesh position={[(min.x + max.x) / 2, (min.y + max.y) / 2, (min.z + max.z) / 2]}>
          <boxGeometry args={[max.x - min.x, max.y - min.y, max.z - min.z]} />
          <meshBasicMaterial color={state.status === 'missing' ? '#d0574a' : '#9aa3ad'} wireframe />
        </mesh>
      )}
    </group>
  );
}

/** The copy with its materials made see-through (drag preview, dimmed) or tinted (selected). */
function withStyle(plain: Object3D, { ghost, selected, dimmed }: ModelStyle): Object3D {
  if (!ghost && !selected && !dimmed) return plain;
  const scene = plain.clone(true);
  scene.traverse((o) => {
    const mesh = o as Mesh;
    if (mesh.isMesh !== true) return;
    const restyle = (m: Material) => {
      const copy = m.clone() as Material & {
        emissive?: { set(c: string): void };
        emissiveIntensity?: number;
      };
      if (ghost || dimmed) {
        copy.transparent = true;
        copy.opacity = ghost ? 0.5 : 0.3;
        copy.depthWrite = false;
      }
      if (selected && copy.emissive !== undefined) {
        copy.emissive.set(SELECTED_EMISSIVE);
        copy.emissiveIntensity = 0.45;
      }
      return copy;
    };
    mesh.material = Array.isArray(mesh.material)
      ? mesh.material.map(restyle)
      : restyle(mesh.material);
  });
  return scene;
}
