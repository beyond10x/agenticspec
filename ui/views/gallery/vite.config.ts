import { fileURLToPath } from 'node:url';
import { defineConfig } from 'vitest/config';
import vue from '@vitejs/plugin-vue';

// The gallery of the block panels: the app generated from ui/views/blocks.ui.yaml (src/generated)
// over ui/kit, answered by the panels' fixtures (ui/views/fixtures) in the browser, with no server.
// The build lands in target/views/gallery, which the gallery's browser test serves.
const here = (p: string) => fileURLToPath(new URL(p, import.meta.url));

export default defineConfig({
  root: here('.'),
  base: './',
  plugins: [vue()],
  resolve: {
    alias: { '@kit': here('../../kit/src/index.ts') },
    dedupe: ['vue'],
  },
  server: {
    host: '127.0.0.1',
    port: 5177,
    strictPort: true,
    fs: { allow: [here('../..')] },
  },
  build: {
    outDir: here('../../../target/views/gallery'),
    emptyOutDir: true,
    reportCompressedSize: false,
    chunkSizeWarningLimit: 1500,
  },
  test: {
    root: here('.'),
    environment: 'happy-dom',
    include: ['tests/**/*.test.ts'],
    testTimeout: 10_000,
  },
});
