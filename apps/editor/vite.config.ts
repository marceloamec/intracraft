import { cpSync, existsSync, readFileSync, statSync } from 'node:fs';
import { resolve, sep } from 'node:path';
import react from '@vitejs/plugin-react';
import { defineConfig, type Plugin } from 'vite';

export default defineConfig({
  plugins: [react(), libraryAssets()],
  worker: { format: 'es' },
});

/** Serves the repo-level library-assets/ folder (downloaded 3D models) at /library-assets/. */
function libraryAssets(): Plugin {
  const root = resolve(import.meta.dirname, '../../library-assets');
  let outDir = '';

  return {
    name: 'library-assets',
    configResolved(config) {
      outDir = resolve(config.root, config.build.outDir);
    },
    configureServer(server) {
      server.middlewares.use('/library-assets', (req, res, next) => {
        const path = resolve(root, decodeURIComponent((req.url ?? '/').split('?')[0]).slice(1));
        if (!path.startsWith(root + sep) || !existsSync(path) || !statSync(path).isFile())
          return next();
        res.setHeader('Content-Type', 'model/gltf-binary');
        res.end(readFileSync(path));
      });
    },
    closeBundle() {
      if (existsSync(root)) cpSync(root, resolve(outDir, 'library-assets'), { recursive: true });
    },
  };
}
