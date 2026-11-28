import { createReadStream, existsSync, statSync } from 'node:fs';
import { createServer } from 'node:http';
import path from 'node:path';
import type { AddressInfo } from 'node:net';

// Serves the built examples (target/views/examples) on a free port of 127.0.0.1 for the whole run
// and hands its address to the tests in EXAMPLES_URL. The examples answer their reads from the
// fixtures it was built with, so nothing else is served.
const types: Record<string, string> = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.svg': 'image/svg+xml',
};

export default async function globalSetup(): Promise<() => Promise<void>> {
  const root = path.resolve(import.meta.dirname, '../../../../../target/views/examples');
  if (!existsSync(path.join(root, 'index.html'))) {
    throw new Error(`${root} has no index.html; build the examples first (task check:examples)`);
  }
  const server = createServer((req, res) => {
    const rel = decodeURIComponent((req.url ?? '/').split('?')[0]).replace(/^\/+/, '') || 'index.html';
    const file = path.resolve(root, rel);
    if (!file.startsWith(root + path.sep) || !existsSync(file) || !statSync(file).isFile()) {
      res.writeHead(404).end();
      return;
    }
    res.writeHead(200, { 'Content-Type': types[path.extname(file)] ?? 'application/octet-stream' });
    createReadStream(file).pipe(res);
  });
  await new Promise<void>((resolve) => server.listen(0, '127.0.0.1', resolve));
  const { port } = server.address() as AddressInfo;
  process.env.EXAMPLES_URL = `http://127.0.0.1:${port}/`;
  return async () => {
    await new Promise<void>((resolve) => server.close(() => resolve()));
  };
}
