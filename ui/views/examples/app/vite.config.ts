import { fileURLToPath } from 'node:url';
import { defineConfig } from 'vitest/config';
import vue from '@vitejs/plugin-vue';

// The derived examples: the apps generated from ui/views/examples/<name>.ui.yaml (src/<name>/generated)
// over ui/kit, answered by the panels' fixtures and the fixtures derived beside each example, in the
// browser, with no server. One page per example; the build lands in target/views/examples.
const here = (p: string) => fileURLToPath(new URL(p, import.meta.url));

export default defineConfig({
  root: here('.'),
  base: './',
  plugins: [vue()],
  resolve: {
    alias: { '@kit': here('../../../kit/src/index.ts') },
    dedupe: ['vue'],
  },
  build: {
    outDir: here('../../../../target/views/examples'),
    emptyOutDir: true,
    reportCompressedSize: false,
    chunkSizeWarningLimit: 1500,
    rollupOptions: {
      input: { index: here('index.html'), workstation: here('workstation.html'), 'large-37': here('large-37.html') },
    },
  },
  test: {
    root: here('.'),
    environment: 'happy-dom',
    include: ['tests/**/*.test.ts'],
    testTimeout: 10_000,
  },
});
