import { spawn } from 'node:child_process';
import path from 'node:path';

// Starts one `agenticstudio serve` for the run: the binary of target/debug, the repository root as
// its root, port 0 so the system picks a free port. Its first line names the address, which the
// tests read from STUDIO_URL. The returned function ends the server after the last test.
export default async function globalSetup(): Promise<() => Promise<void>> {
  const root = path.resolve(__dirname, '../../..');
  const binary = path.join(root, 'target', 'debug', 'agenticstudio');
  const server = spawn(binary, ['serve', '--root', root, '--port', '0'], {
    stdio: ['ignore', 'pipe', 'inherit'],
  });
  const url = await new Promise<string>((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error('agenticstudio named no address within 10 s')), 10_000);
    let seen = '';
    server.stdout.on('data', (chunk: Buffer) => {
      seen += chunk.toString();
      const m = seen.match(/serving (http:\/\/127\.0\.0\.1:\d+\/)/);
      if (m) {
        clearTimeout(timer);
        resolve(m[1]);
      }
    });
    server.on('error', (e) => {
      clearTimeout(timer);
      reject(new Error(`cannot start ${binary}: ${e.message}; run task check:studio, which builds it`));
    });
    server.on('exit', (code) => {
      clearTimeout(timer);
      reject(new Error(`agenticstudio ended with ${code} before it named its address`));
    });
  });
  process.env.STUDIO_URL = url;
  return async () => {
    server.kill();
  };
}
