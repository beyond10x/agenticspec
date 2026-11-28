import { fileURLToPath } from 'node:url';
import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';

// The generic operator app, built into ui/operator/built (or the directory OPERATOR_OUT names) as
// three files with fixed names, index.html, app.js and app.css, which the agenticspec binary
// embeds. One chunk, no hashes in names and every asset inlined, so the file list never changes.
const here = (p: string) => fileURLToPath(new URL(p, import.meta.url));

export default defineConfig({
  root: here('.'),
  base: './',
  plugins: [vue()],
  resolve: {
    alias: { '@kit': here('../kit/src/index.ts') },
    dedupe: ['vue'],
  },
  define: { 'process.env.NODE_ENV': JSON.stringify('production') },
  build: {
    outDir: process.env.OPERATOR_OUT ?? here('built'),
    emptyOutDir: true,
    assetsDir: '',
    assetsInlineLimit: Number.MAX_SAFE_INTEGER,
    cssCodeSplit: false,
    modulePreload: false,
    reportCompressedSize: false,
    chunkSizeWarningLimit: 1500,
    rollupOptions: {
      output: {
        entryFileNames: 'app.js',
        chunkFileNames: 'app-[name].js',
        assetFileNames: (info) => (info.names?.some((n) => n.endsWith('.css')) ? 'app.css' : '[name][extname]'),
        codeSplitting: false,
      },
    },
  },
});
