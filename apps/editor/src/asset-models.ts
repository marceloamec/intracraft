// GLB models of the asset catalogue (A16): each file is loaded once (GLTFLoader + meshopt) and cloned
// for every place that draws it, so each copy has its own pose and style. Files come from
// /library-assets/ (`pnpm assets:fetch`); a missing file is reported, not thrown.
import { builtInCatalog } from '@intracraft/asset-catalog';
import { useEffect, useMemo, useState } from 'react';
import type { Object3D } from 'three';
import { MeshoptDecoder } from 'three/examples/jsm/libs/meshopt_decoder.module.js';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';

export type ModelState =
  { status: 'loading' } | { status: 'missing' } | { status: 'ready'; scene: Object3D };

const loader = new GLTFLoader().setMeshoptDecoder(MeshoptDecoder);
const requests = new Map<string, Promise<Object3D>>();
const loaded = new Map<string, Object3D>();

/** A copy of the model at `url`, loaded once; the state changes when the file arrives. */
export function useModel(url: string): ModelState {
  const [state, setState] = useState<{ url: string; scene: Object3D | 'missing' } | null>(() => {
    const scene = loaded.get(url);
    return scene === undefined ? null : { url, scene };
  });
  useEffect(() => {
    let cancelled = false;
    request(url).then(
      (scene) => !cancelled && setState({ url, scene }),
      () => !cancelled && setState({ url, scene: 'missing' }),
    );
    return () => {
      cancelled = true;
    };
  }, [url]);
  const current = state?.url === url ? state.scene : undefined;
  return useMemo(() => {
    if (current === undefined) return { status: 'loading' };
    if (current === 'missing') return { status: 'missing' };
    return { status: 'ready', scene: current.clone(true) };
  }, [current]);
}

export function useAssetModel(assetId: string): ModelState {
  return useModel(builtInCatalog.urlOf(assetId));
}

/**
 * A new copy of an asset's model if its file has already arrived; otherwise starts loading it and
 * returns undefined. For code that draws outside React (the running simulation's pallets and objects).
 */
export function assetCopy(assetId: string): Object3D | undefined {
  const url = builtInCatalog.urlOf(assetId);
  const scene = loaded.get(url);
  if (scene !== undefined) return scene.clone(true);
  request(url).catch(() => undefined);
  return undefined;
}

/** True once an asset's file has arrived; starts loading it otherwise. */
export function isAssetLoaded(assetId: string): boolean {
  const url = builtInCatalog.urlOf(assetId);
  if (loaded.has(url)) return true;
  request(url).catch(() => undefined);
  return false;
}

function request(url: string): Promise<Object3D> {
  let pending = requests.get(url);
  if (pending === undefined) {
    pending = loader.loadAsync(url).then((gltf) => {
      loaded.set(url, gltf.scene);
      return gltf.scene;
    });
    requests.set(url, pending);
  }
  return pending;
}
