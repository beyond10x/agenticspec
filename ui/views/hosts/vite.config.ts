import { fileURLToPath } from 'node:url';
import { defineConfig } from 'vite';

// The React host page of the custom elements' browser test, built into target/views/hosts. The
// plain page (plain.html) is served as written; both load the bundle from /elements/.
const here = (p: string) => fileURLToPath(new URL(p, import.meta.url));

export default defineConfig({
  root: here('.'),
  base: '/hosts/',
  build: {
    outDir: here('../../../target/views/hosts'),
    emptyOutDir: true,
    reportCompressedSize: false,
    chunkSizeWarningLimit: 1500,
    rollupOptions: { input: { react: here('react.html') } },
  },
});
