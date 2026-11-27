import {useMemo, useRef, useState, type ReactNode} from 'react';
import Link from '@docusaurus/Link';
import data from '@site/src/generated/kpis.json';
import {useScrollRegion} from './scrollRegion';

interface Row {
  id: string;
  block: string;
  layer: number | null;
  family: string;
  planned: boolean;
  route: string;
  kind: string;
  unit: string;
  summary: string;
  how: string;
  direction: string;
  modes: {mode: string; direction: string}[];
  gate: boolean;
  reads: string[];
  status: string;
  source: string;
}

const rows = (data as unknown as {kpis: Row[]}).kpis;

const KINDS = ['correctness', 'completeness', 'quality'];
const STATUSES = ['computed', 'simulated', 'both', 'planned'];
const STATUS_WORDS: Record<string, string> = {
  computed: 'computed today',
  simulated: 'simulated',
  both: 'computed and simulated',
  planned: 'planned',
};

type Key = 'id' | 'block' | 'kind' | 'unit' | 'direction' | 'gate' | 'status';
const COLUMNS: [Key, string][] = [
  ['id', 'KPI'],
  ['block', 'block'],
  ['kind', 'kind'],
  ['unit', 'unit'],
  ['direction', 'better'],
  ['gate', 'gate'],
  ['status', 'status'],
];

function layerOf(r: Row): number {
  return r.layer ?? 99;
}

function compare(a: Row, b: Row, key: Key): number {
  switch (key) {
    case 'block':
      return layerOf(a) - layerOf(b) || a.block.localeCompare(b.block) || a.id.localeCompare(b.id);
    case 'kind':
      return KINDS.indexOf(a.kind) - KINDS.indexOf(b.kind) || a.id.localeCompare(b.id);
    case 'status':
      return STATUSES.indexOf(a.status) - STATUSES.indexOf(b.status) || a.id.localeCompare(b.id);
    case 'gate':
      return Number(b.gate) - Number(a.gate) || a.id.localeCompare(b.id);
    default:
      return String(a[key]).localeCompare(String(b[key])) || a.id.localeCompare(b.id);
  }
}

function toggle(set: string[], value: string): string[] {
  return set.includes(value) ? set.filter((v) => v !== value) : [...set, value];
}

function Chips({label, values, words, selected, onChange, counts}: {
  label: string;
  values: string[];
  words?: Record<string, string>;
  selected: string[];
  onChange: (next: string[]) => void;
  counts: Record<string, number>;
}): ReactNode {
  return (
    <div className="as-kpis__group" role="group" aria-label={label}>
      <span>{label}</span>
      {values.map((v) => (
        <button key={v} type="button" className="as-chip" aria-pressed={selected.includes(v)} onClick={() => onChange(toggle(selected, v))}>
          {words?.[v] ?? v}
          <small>{counts[v] ?? 0}</small>
        </button>
      ))}
    </div>
  );
}

/** Every KPI of the catalogue in one table: filter by block, kind, status and gate, search, sort. */
export default function KpiCatalogue(): ReactNode {
  const [query, setQuery] = useState('');
  const [block, setBlock] = useState('');
  const [kinds, setKinds] = useState<string[]>([]);
  const [statuses, setStatuses] = useState<string[]>([]);
  const [gate, setGate] = useState<string[]>([]);
  const [sort, setSort] = useState<{key: Key; up: boolean}>({key: 'block', up: true});
  const scroller = useRef<HTMLDivElement>(null);
  const region = useScrollRegion(scroller, 'KPI table');

  const blocks = useMemo(() => {
    const seen = new Map<string, Row>();
    for (const r of rows) if (!seen.has(r.block)) seen.set(r.block, r);
    return [...seen.values()].sort((a, b) => compare(a, b, 'block')).map((r) => r);
  }, []);

  const shown = useMemo(() => {
    const q = query.trim().toLowerCase();
    const out = rows.filter(
      (r) =>
        (!block || r.block === block) &&
        (kinds.length === 0 || kinds.includes(r.kind)) &&
        (statuses.length === 0 || statuses.includes(r.status)) &&
        (gate.length === 0 || gate.includes(r.gate ? 'gate' : 'no gate')) &&
        (!q || `${r.id} ${r.summary} ${r.how} ${r.block}`.toLowerCase().includes(q)),
    );
    out.sort((a, b) => (sort.up ? 1 : -1) * compare(a, b, sort.key));
    return out;
  }, [query, block, kinds, statuses, gate, sort]);

  const count = (pick: (r: Row) => string) => {
    const c: Record<string, number> = {};
    for (const r of rows) c[pick(r)] = (c[pick(r)] ?? 0) + 1;
    return c;
  };

  if (rows.length === 0) {
    return <p className="as-kpis__empty">The KPI catalogue holds no KPI yet.</p>;
  }
  return (
    <section className="as-kpis" aria-label="KPI catalogue">
      <div className="as-kpis__controls">
        <label className="as-kpis__field">
          Search
          <input type="search" value={query} placeholder="name, what it measures, how" onChange={(e) => setQuery(e.target.value)} />
        </label>
        <label className="as-kpis__field">
          Block
          <select value={block} onChange={(e) => setBlock(e.target.value)}>
            <option value="">every block</option>
            {blocks.map((r) => (
              <option key={r.block} value={r.block}>
                {r.planned ? `${r.block} (planned)` : `${r.layer} · ${r.block}`}
              </option>
            ))}
          </select>
        </label>
        <div className="as-kpis__chips">
          <Chips label="Kind" values={KINDS} selected={kinds} onChange={setKinds} counts={count((r) => r.kind)} />
          <Chips label="Status" values={STATUSES} selected={statuses} onChange={setStatuses} counts={count((r) => r.status)} />
          <Chips label="Gate" values={['gate', 'no gate']} selected={gate} onChange={setGate} counts={count((r) => (r.gate ? 'gate' : 'no gate'))} />
        </div>
      </div>
      <p className="as-kpis__count" aria-live="polite">
        {shown.length} of {rows.length} KPIs
      </p>
      <div className="as-kpis__scroll" ref={scroller} {...region}>
        <table>
          <thead>
            <tr>
              {COLUMNS.map(([key, label]) => (
                <th key={key} aria-sort={sort.key === key ? (sort.up ? 'ascending' : 'descending') : 'none'}>
                  <button type="button" onClick={() => setSort({key, up: sort.key === key ? !sort.up : true})}>
                    {label}
                    {sort.key === key ? (sort.up ? ' ▲' : ' ▼') : ''}
                  </button>
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {shown.map((r) => (
              <tr key={r.id}>
                <td className="as-kpis__what">
                  <code>{r.id}</code>
                  <p>{r.summary}</p>
                  {r.how && <small>How: {r.how}</small>}
                  {r.modes.length > 0 && (
                    <small>By work mode: {r.modes.map((m) => `${m.mode} ${m.direction}`).join(', ')}</small>
                  )}
                  {r.source && <small>Source: {r.source}</small>}
                </td>
                <td>
                  <Link to={r.route} className={`as-kpis__block as-fam-${r.family}`}>
                    <i />
                    {r.block}
                  </Link>
                  {r.planned && (
                    <>
                      {' '}
                      <span className="as-tag as-tag--planned">planned</span>
                    </>
                  )}
                </td>
                <td>{r.kind}</td>
                <td>{r.unit}</td>
                <td>{r.direction}</td>
                <td>{r.gate ? <span className="as-tag as-tag--gate">gate</span> : '–'}</td>
                <td>{STATUS_WORDS[r.status] ?? r.status}</td>
              </tr>
            ))}
          </tbody>
        </table>
        {shown.length === 0 && <p className="as-kpis__empty">No KPI matches these filters.</p>}
      </div>
    </section>
  );
}
