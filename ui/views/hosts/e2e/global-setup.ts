import { createReadStream, existsSync, statSync } from 'node:fs';
import { createServer, type ServerResponse } from 'node:http';
import path from 'node:path';
import type { AddressInfo } from 'node:net';

// One server for the whole run, on a free port of 127.0.0.1, handed to the tests in HOSTS_URL:
// the bundle of the custom elements (/elements/, target/views/elements), the host pages (/hosts/:
// plain.html as written, the React page from target/views/hosts) and a fixture source
// (/fixtures/): a view's rows from ui/views/fixtures, a layout from ui/views/examples, and an empty
// answer for every channel.
const ui = path.resolve(import.meta.dirname, '../../..');
const target = path.resolve(ui, '../target/views');
const types: Record<string, string> = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.json': 'application/json',
  '.svg': 'image/svg+xml',
};

function send(res: ServerResponse, file: string): void {
  if (!existsSync(file) || !statSync(file).isFile()) {
    res.writeHead(404).end();
    return;
  }
  res.writeHead(200, { 'Content-Type': types[path.extname(file)] ?? 'application/octet-stream' });
  createReadStream(file).pipe(res);
}

function under(root: string, rel: string): string {
  const file = path.resolve(root, rel);
  return file.startsWith(root + path.sep) ? file : path.join(root, '..missing');
}

export default async function globalSetup(): Promise<() => Promise<void>> {
  for (const f of ['elements/agentic-views.js', 'hosts/react.html']) {
    if (!existsSync(path.join(target, f))) throw new Error(`${target}/${f} is missing; build it first (task check:elements)`);
  }
  const server = createServer((req, res) => {
    const url = decodeURIComponent((req.url ?? '/').split('?')[0]);
    let m: RegExpMatchArray | null;
    if ((m = url.match(/^\/elements\/(.+)$/))) return send(res, under(path.join(target, 'elements'), m[1]));
    if (url === '/hosts/plain.html') return send(res, path.join(ui, 'views/hosts/plain.html'));
    if ((m = url.match(/^\/hosts\/(.+)$/))) return send(res, under(path.join(target, 'hosts'), m[1]));
    if ((m = url.match(/^\/fixtures\/api\/views\/([\w.]+)$/))) return send(res, path.join(ui, 'views/fixtures', `${m[1]}.json`));
    if ((m = url.match(/^\/fixtures\/layouts\/([\w-]+)\.json$/))) return send(res, path.join(ui, 'views/examples', `${m[1]}.layout.json`));
    if (url.startsWith('/fixtures/api/channels/')) return void res.writeHead(204).end();
    res.writeHead(404).end();
  });
  await new Promise<void>((resolve) => server.listen(0, '127.0.0.1', resolve));
  const { port } = server.address() as AddressInfo;
  process.env.HOSTS_URL = `http://127.0.0.1:${port}`;
  return async () => {
    await new Promise<void>((resolve) => server.close(() => resolve()));
  };
}
