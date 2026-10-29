// Orbitable preview of one library model. Loads the GLB, centres it and frames the camera on it.
import { Canvas, useThree } from '@react-three/fiber';
import { useEffect, useState } from 'react';
import { Box3, type Object3D, Sphere, Vector3 } from 'three';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';
import { MeshoptDecoder } from 'three/examples/jsm/libs/meshopt_decoder.module.js';
import { useT } from '../i18n/useT';

type LoadState = { status: 'loading' } | { status: 'error' } | { status: 'ready'; model: Object3D };

export function AssetPreview({ url }: { url: string }) {
  const t = useT();
  const [loaded, setLoaded] = useState<{ url: string; state: LoadState } | null>(null);
  const state: LoadState = loaded?.url === url ? loaded.state : { status: 'loading' };

  useEffect(() => {
    let cancelled = false;
    new GLTFLoader().setMeshoptDecoder(MeshoptDecoder).load(
      url,
      (gltf) => !cancelled && setLoaded({ url, state: { status: 'ready', model: gltf.scene } }),
      undefined,
      () => !cancelled && setLoaded({ url, state: { status: 'error' } }),
    );
    return () => {
      cancelled = true;
    };
  }, [url]);

  return (
    <div className="asset-preview">
      <Canvas camera={{ fov: 40, near: 0.01, far: 1000 }}>
        <ambientLight intensity={0.9} />
        <directionalLight position={[3, 5, 4]} intensity={1.8} />
        <directionalLight position={[-3, 2, -4]} intensity={0.6} />
        {state.status === 'ready' && <FramedModel model={state.model} />}
      </Canvas>
      {state.status !== 'ready' && (
        <div className="asset-preview-status">
          {state.status === 'loading' ? t('loadingModel') : t('modelMissing')}
        </div>
      )}
    </div>
  );
}

function FramedModel({ model }: { model: Object3D }) {
  const camera = useThree((s) => s.camera);
  const domElement = useThree((s) => s.gl.domElement);

  useEffect(() => {
    const sphere = new Box3().setFromObject(model).getBoundingSphere(new Sphere());
    const direction = new Vector3(1, 0.7, 1).normalize();
    camera.position.copy(sphere.center).addScaledVector(direction, sphere.radius * 2.6);

    const controls = new OrbitControls(camera, domElement);
    controls.target.copy(sphere.center);
    controls.autoRotate = true;
    controls.autoRotateSpeed = 2;
    controls.update();
    // OrbitControls only advances autoRotate when update() is called.
    const timer = window.setInterval(() => controls.update(), 16);
    return () => {
      window.clearInterval(timer);
      controls.dispose();
    };
  }, [model, camera, domElement]);

  return <primitive object={model} />;
}
