import { createReadStream, existsSync, statSync } from 'node:fs';
import { createServer, type IncomingMessage, type ServerResponse } from 'node:http';
import path from 'node:path';
import type { AddressInfo } from 'node:net';

// One server for the whole run, on a free port of 127.0.0.1, handed to the tests in OPERATOR_URL:
// the directory `agenticspec ui build` wrote (target/operator/workstation, or OPERATOR_DIR) at /,
// and a fixture adapter of the runtime interface at /api/ (the default base path of config.json):
// R1 answers every view from the panels' fixtures (ui/views/fixtures) at any scope, R2 answers
// 204, R4 answers `{"row": {}}`, and R5 answers 404 (no action set, every action shows) unless
// the request carries the cookie `r5=deny`, which withholds every command. GET /seen lists the
// commands it was sent.
const ui = path.resolve(import.meta.dirname, '../..');
const dir = path.resolve(ui, '..', process.env.OPERATOR_DIR ?? 'target/operator/workstation');
const fixtures = path.join(ui, 'views/fixtures');
const types: Record<string, string> = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.json': 'application/json',
};

function send(res: ServerResponse, file: string): void {
  if (!existsSync(file) || !statSync(file).isFile()) {
    res.writeHead(404, { 'Content-Type': 'application/json' }).end('{"error":"NotFound","message":""}');
    return;
  }
  res.writeHead(200, { 'Content-Type': types[path.extname(file)] ?? 'application/octet-stream' });
  createReadStream(file).pipe(res);
}

function json(res: ServerResponse, status: number, body: unknown): void {
  res.writeHead(status, { 'Content-Type': 'application/json' }).end(JSON.stringify(body));
}

export default async function globalSetup(): Promise<() => Promise<void>> {
  if (!existsSync(path.join(dir, 'config.json'))) throw new Error(`${dir} holds no operator UI; write it first (task check:operator)`);
  const seen: { scope: string; command: string }[] = [];
  const server = createServer((req: IncomingMessage, res: ServerResponse) => {
    const url = decodeURIComponent((req.url ?? '/').split('?')[0]);
    let m: RegExpMatchArray | null;
    if (url === '/seen') return json(res, 200, seen);
    if ((m = url.match(/^\/api\/scopes\/(.+)\/api\/views\/([\w.]+)$/))) return send(res, path.join(fixtures, `${m[2]}.json`));
    if (url.match(/^\/api\/scopes\/.+\/api\/channels\//)) return void res.writeHead(204).end();
    if ((m = url.match(/^\/api\/scopes\/(.+)\/api\/actions$/))) {
      if (!(req.headers.cookie ?? '').includes('r5=deny')) return json(res, 404, { error: 'NotFound', message: 'no action set' });
      return json(res, 200, [{ command: 'agentic.channel.SendMessage', allowed: false }]);
    }
    if ((m = url.match(/^\/api\/scopes\/(.+)\/api\/commands\/([\w.]+)$/)) && req.method === 'POST') {
      seen.push({ scope: m[1], command: m[2] });
      return json(res, 200, { row: {} });
    }
    if (url.startsWith('/api/')) return json(res, 404, { error: 'NotFound', message: url });
    const rel = url.replace(/^\/+/, '') || 'index.html';
    const file = path.resolve(dir, rel);
    return send(res, file.startsWith(dir + path.sep) ? file : path.join(dir, '..missing'));
  });
  await new Promise<void>((resolve) => server.listen(0, '127.0.0.1', resolve));
  const { port } = server.address() as AddressInfo;
  process.env.OPERATOR_URL = `http://127.0.0.1:${port}`;
  return async () => {
    await new Promise<void>((resolve) => server.close(() => resolve()));
  };
}
