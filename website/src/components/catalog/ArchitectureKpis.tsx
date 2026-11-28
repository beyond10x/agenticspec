import {useMemo, useRef, useState, type ReactNode} from 'react';
import Link from '@docusaurus/Link';
import {useScrollRegion} from '../blocks/scrollRegion';
import './catalog.css';

interface Kpi {
  id: string;
  what: string;
}

interface Row {
  name: string;
  to: string;
  values: {value: number; text: string}[];
}

/** The column a table is sorted by: the architecture's name, or the index of a KPI. */
type Key = 'name' | number;

function compare(a: Row, b: Row, key: Key): number {
  if (key === 'name') return a.name.localeCompare(b.name);
  return a.values[key].value - b.values[key].value || a.name.localeCompare(b.name);
}

/**
 * Every architecture with the KPIs of the architecture as a whole, one column per KPI, sorted by
 * any column; written by agenticspec-docs from what `agenticspec sim check` reports.
 */
export default function ArchitectureKpis({data}: {data: {kpis: Kpi[]; rows: Row[]}}): ReactNode {
  const [sort, setSort] = useState<{key: Key; up: boolean}>({key: 'name', up: true});
  const scroller = useRef<HTMLDivElement>(null);
  const region = useScrollRegion(scroller, 'Architectures as a whole');
  const rows = useMemo(
    () => [...data.rows].sort((a, b) => (sort.up ? 1 : -1) * compare(a, b, sort.key)),
    [data.rows, sort],
  );
  const columns: [Key, string, string][] = [
    ['name', 'architecture', 'The architecture; its catalog entry, or its page when no experiment searches it'],
    ...data.kpis.map((k, i): [Key, string, string] => [i, k.id, k.what]),
  ];
  return (
    <section className="as-kpis as-cat-whole" aria-label="Architectures as a whole">
      <div className="as-kpis__scroll" ref={scroller} {...region}>
        <table>
          <thead>
            <tr>
              {columns.map(([key, label, what]) => (
                <th key={label} title={what} aria-sort={sort.key === key ? (sort.up ? 'ascending' : 'descending') : 'none'}>
                  <button type="button" onClick={() => setSort({key, up: sort.key === key ? !sort.up : true})}>
                    {label}
                    {sort.key === key ? (sort.up ? ' ▲' : ' ▼') : ''}
                  </button>
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {rows.map((r) => (
              <tr key={r.name}>
                <td>
                  <Link to={r.to}>{r.name}</Link>
                </td>
                {r.values.map((v, i) => (
                  <td key={data.kpis[i].id} className="as-cat-whole__num">
                    {v.text}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </section>
  );
}
