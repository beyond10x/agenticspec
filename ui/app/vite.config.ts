import { fileURLToPath } from 'node:url';
import { defineConfig } from 'vitest/config';
import vue from '@vitejs/plugin-vue';

// The build lands in target/ui/www, which `agenticspec serve` serves; `npm run dev` proxies the
// views and channels to a running `agenticspec serve`.
const outDir = fileURLToPath(new URL('../../target/ui/www', import.meta.url));

export default defineConfig({
  base: '/',
  plugins: [vue()],
  server: {
    host: '127.0.0.1',
    port: 5175,
    strictPort: true,
    proxy: { '/view/': { target: 'http://127.0.0.1:7420' } },
  },
  build: {
    outDir,
    emptyOutDir: true,
    reportCompressedSize: false,
  },
  test: {
    environment: 'happy-dom',
    include: ['src/**/*.test.ts'],
    testTimeout: 10_000,
  },
});
