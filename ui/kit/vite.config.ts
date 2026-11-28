import { fileURLToPath } from 'node:url';
import { defineConfig } from 'vitest/config';
import vue from '@vitejs/plugin-vue';

// The gallery of ui/kit (every component in both themes) builds into target/ui-kit/gallery, which
// the gallery's browser test serves. vitest runs the kit's unit tests in happy-dom.
const here = (p: string) => fileURLToPath(new URL(p, import.meta.url));

export default defineConfig({
  root: here('.'),
  base: './',
  plugins: [vue()],
  resolve: { alias: { '@kit': here('./src/index.ts') } },
  build: {
    outDir: here('../../target/ui-kit/gallery'),
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
