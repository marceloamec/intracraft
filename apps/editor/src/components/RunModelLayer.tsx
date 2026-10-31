// Catalogue models of a running 3D simulation: the pallet and load models a source creates (S09 §1.1)
// and the objects dropped into the run (S17 §4). They are built outside React, one group per pallet
// or object, and placed every frame at the pose the physics reports. A pallet is drawn here only once
// all its models have loaded; until then PalletLayer draws it as boxes (see `hasModels`).
import type { Pose } from '@intracraft/sim-physics';
import { useFrame } from '@react-three/fiber';
import { useMemo, useRef } from 'react';
import { Group } from 'three';
import { assetCopy, isAssetLoaded } from '../asset-models';
import { useLayoutStore } from '../layout-store';
import { liveFrame } from '../sim-store';
import type { SimSnapshot } from '../sim/protocol';

type Look = NonNullable<SimSnapshot['looks']>[number];

export function RunModelLayer() {
  const root = useRef<Group>(null);
  const groups = useRef(new Map<string, Group>());
  const layout = useLayoutStore((s) => s.layout);
  const placed = useMemo(() => new Set(layout.instances.map((i) => i.id)), [layout]);

  useFrame(() => {
    const parent = root.current;
    if (parent === null) return;
    const snapshot = liveFrame.snapshot;
    const wanted = new Map<string, { pose: Pose; build: () => Group | undefined }>();
    for (const [id, look] of Object.entries(snapshot?.looks ?? {})) {
      const pose = snapshot?.poses?.[Number(id)];
      if (pose !== undefined) wanted.set(`pallet:${id}`, { pose, build: () => lookGroup(look) });
    }
    for (const [id, view] of Object.entries(snapshot?.objects ?? {})) {
      // Placed objects are drawn by their own model (AssetModel); only live ones are drawn here.
      if (placed.has(id)) continue;
      wanted.set(`object:${id}`, { pose: view.pose, build: () => modelGroup(view.asset) });
    }
    for (const [key, group] of groups.current) {
      if (wanted.has(key)) continue;
      parent.remove(group);
      groups.current.delete(key);
    }
    for (const [key, { pose, build }] of wanted) {
      let group = groups.current.get(key);
      if (group === undefined) {
        group = build();
        if (group === undefined) continue;
        groups.current.set(key, group);
        parent.add(group);
      }
      group.position.set(pose.position.x, pose.position.y, pose.position.z);
      group.quaternion.set(pose.rotation.x, pose.rotation.y, pose.rotation.z, pose.rotation.w);
    }
  });

  return <group ref={root} userData={{ noPick: true }} />;
}

/** True when every model of the look has loaded, so the pallet is drawn with models, not boxes. */
export function hasModels(look: Look | undefined): boolean {
  if (look === undefined) return false;
  return [look.pallet, look.load].every((part) => part === undefined || isAssetLoaded(part.asset));
}

/** Pallet and load models placed in the pallet's body frame; undefined while one is loading. */
function lookGroup(look: Look): Group | undefined {
  const parts = [look.pallet, look.load].filter((part) => part !== undefined);
  const group = new Group();
  for (const part of parts) {
    const model = assetCopy(part.asset);
    if (model === undefined) return undefined;
    model.rotation.y = part.placement.yaw;
    const t = part.placement.translation;
    model.position.set(t.x, t.y, t.z);
    group.add(model);
  }
  return group;
}

function modelGroup(asset: string): Group | undefined {
  const model = assetCopy(asset);
  if (model === undefined) return undefined;
  const group = new Group();
  group.add(model);
  return group;
}
