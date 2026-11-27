// A static host with the layout of GitHub Pages, for the checks that must hold on the published
// site as well as under `docusaurus serve`: website/build answered under the base path, with no
// rewrite and no clean-URL redirect. A request path is answered with the file of that path; else,
// without a trailing slash, with the same path plus `.html` (the order the trailing-slash copies of
// the docs-system assume, so that `docs/x` is `docs/x.html`); else a directory is sent on to the
// path with a slash (301) and answered with its index.html; anything else is a 404.

import {existsSync, readFileSync, statSync} from 'node:fs';
import {createServer} from 'node:http';
import type {AddressInfo} from 'node:net';
import {extname, join, normalize} from 'node:path';
import {adapter, ROOT} from './site';

const TYPES: Record<string, string> = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.css': 'text/css; charset=utf-8',
  '.json': 'application/json',
  '.svg': 'image/svg+xml',
  '.png': 'image/png',
  '.ico': 'image/x-icon',
  '.xml': 'application/xml',
  '.txt': 'text/plain; charset=utf-8',
  '.woff2': 'font/woff2',
};

export interface Host {
  origin: string;
  close: () => Promise<void>;
}

export async function pagesHost(): Promise<Host> {
  const dir = `${ROOT}website/build`;
  const base = `${adapter.base_path}/`;
  const isFile = (p: string) => existsSync(p) && statSync(p).isFile();
  const server = createServer((req, res) => {
    const u = new URL(req.url ?? '/', 'http://host');
    if (!u.pathname.startsWith(base)) {
      res.writeHead(404).end();
      return;
    }
    const rel = normalize(decodeURIComponent(u.pathname.slice(base.length))).replace(/^(\.\.(\/|$))+/, '');
    const p = join(dir, rel);
    let file = '';
    if (isFile(p)) file = p;
    else if (!u.pathname.endsWith('/') && isFile(`${p}.html`)) file = `${p}.html`;
    else if (existsSync(p) && statSync(p).isDirectory()) {
      if (!u.pathname.endsWith('/')) {
        res.writeHead(301, {Location: `${u.pathname}/${u.search}`}).end();
        return;
      }
      if (isFile(join(p, 'index.html'))) file = join(p, 'index.html');
    }
    if (!file) {
      res.writeHead(404).end();
      return;
    }
    res.writeHead(200, {'Content-Type': TYPES[extname(file)] ?? 'application/octet-stream'}).end(readFileSync(file));
  });
  await new Promise<void>((done) => server.listen(0, '127.0.0.1', done));
  const {port} = server.address() as AddressInfo;
  return {origin: `http://127.0.0.1:${port}`, close: () => new Promise((done) => server.close(() => done()))};
}
