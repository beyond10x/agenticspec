import { fileURLToPath } from 'node:url';
import { defineConfig } from 'vitest/config';
import vue from '@vitejs/plugin-vue';

// The studio app: the pages generated from studio/studio.ui.yaml (src/generated), the host slots
// (src/slots) and ui/kit. The build lands in target/studio/www, which `agenticstudio serve`
// serves; `npm run dev` proxies /api to a running `agenticstudio serve` on its default port.
const here = (p: string) => fileURLToPath(new URL(p, import.meta.url));

export default defineConfig({
  root: here('.'),
  base: './',
  plugins: [vue()],
  resolve: {
    alias: { '@kit': here('../kit/src/index.ts') },
    dedupe: ['vue'],
  },
  server: {
    host: '127.0.0.1',
    port: 5176,
    strictPort: true,
    proxy: { '/api/': { target: 'http://127.0.0.1:7431' } },
  },
  build: {
    outDir: here('../../target/studio/www'),
    emptyOutDir: true,
    reportCompressedSize: false,
  },
  test: {
    root: here('.'),
    environment: 'happy-dom',
    include: ['tests/**/*.test.ts'],
    testTimeout: 10_000,
  },
});
