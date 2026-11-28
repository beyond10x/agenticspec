import { fileURLToPath } from 'node:url';
import { defineConfig } from 'vitest/config';
import vue from '@vitejs/plugin-vue';

// The package of ui/views: its unit tests (tests/), and the bundle of its custom elements
// (src/elements.ts, Vue included) with the kit's stylesheet into target/views/elements as
// agentic-views.js and agentic-views.css.
const here = (p: string) => fileURLToPath(new URL(p, import.meta.url));

export default defineConfig({
  root: here('.'),
  plugins: [vue()],
  resolve: {
    alias: { '@kit': here('../kit/src/index.ts') },
    dedupe: ['vue'],
  },
  define: { 'process.env.NODE_ENV': JSON.stringify('production') },
  build: {
    outDir: here('../../target/views/elements'),
    emptyOutDir: true,
    reportCompressedSize: false,
    chunkSizeWarningLimit: 1500,
    cssCodeSplit: false,
    lib: {
      entry: here('src/elements.ts'),
      formats: ['es'],
      fileName: () => 'agentic-views.js',
      cssFileName: 'agentic-views',
    },
  },
  test: {
    root: here('.'),
    environment: 'happy-dom',
    include: ['tests/**/*.test.ts'],
    testTimeout: 10_000,
  },
});
