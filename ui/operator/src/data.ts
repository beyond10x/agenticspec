// The data files `agenticspec ui build` writes beside the app: `config.json` (the architecture,
// the base path of the runtime's adapter), `views.json` (the layout of the derived UI,
// `agenticspec-ui-layout/1`), `contract.json` (`agenticspec-contract/1`) and `ports.json`
// (`agenticspec-ports/1`). The app reads them at start; nothing of an architecture is built in.
import type { Layout } from '../../views/src/index';

export interface Config {
  format: 'agenticspec-operator-config/1';
  /** The architecture id of the scopes the app reads at, `{api_base}/scopes/{architecture}/...`. */
  architecture: string;
  /** The base path (or URL) of the runtime's adapter, the `{root}` of the runtime interface. */
  api_base: string;
  title: string;
}

export interface ContractPort {
  port: string;
  part: string;
  protocol: string;
  instances: number;
}

export interface Contract {
  format: 'agenticspec-contract/1';
  architecture: string;
  ports: ContractPort[];
}

export interface CatalogPort {
  name: string;
  block: string;
  kind: string;
}

export interface Ports {
  format: string;
  ports: CatalogPort[];
}

export interface Data {
  config: Config;
  layout: Layout;
  contract: Contract | null;
  ports: Ports | null;
}

async function file<T>(name: string, required: boolean): Promise<T | null> {
  const r = await fetch(`./${name}`, { cache: 'no-cache' });
  if (r.ok) return (await r.json()) as T;
  if (required || r.status !== 404) throw new Error(`${name}: ${r.status}`);
  return null;
}

/** Reads the data files; `config.json` and `views.json` must be there. */
export async function loadData(): Promise<Data> {
  const [config, layout, contract, ports] = await Promise.all([
    file<Config>('config.json', true),
    file<Layout>('views.json', true),
    file<Contract>('contract.json', false),
    file<Ports>('ports.json', false),
  ]);
  if (config?.format !== 'agenticspec-operator-config/1') throw new Error('config.json: not an agenticspec-operator-config/1 document');
  if (layout?.format !== 'agenticspec-ui-layout/1') throw new Error('views.json: not an agenticspec-ui-layout/1 document');
  return { config, layout, contract, ports };
}

/** `/api/` is `/api`: the client appends `/scopes/...` to it. */
export function apiRoot(config: Config): string {
  return config.api_base.replace(/\/+$/, '');
}
