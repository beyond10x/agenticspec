import {useMemo, useRef, useState, type ReactNode} from 'react';
import Link from '@docusaurus/Link';
import data from '@site/src/generated/blocks.json';
import {FAMILIES, iconPath} from './families';
import {useScrollRegion} from './scrollRegion';

interface MapBlock {
  name: string;
  layer: number;
  family: string;
  summary: string;
  route: string;
  kpis: number;
  entities: number;
  commands: number;
  events: number;
  views: number;
  uses: string[];
  usedBy: string[];
  x: number;
  y: number;
  w: number;
  h: number;
}

interface MapEdge {
  from: string;
  to: string;
  count: number;
  d: string;
}

interface MapRow {
  layer: number;
  name: string;
  y: number;
  h: number;
}

interface MapData {
  width: number;
  height: number;
  rows: MapRow[];
  blocks: MapBlock[];
  edges: MapEdge[];
}

const map = data as unknown as MapData;

function Icon({block, className}: {block: string; className: string}): ReactNode {
  return (
    <svg className={className} viewBox="0 0 14 14" aria-hidden="true">
      <path d={iconPath(block)} />
    </svg>
  );
}

function plural(n: number, word: string): string {
  return `${n} ${word}${n === 1 ? '' : 's'}`;
}

/** The layer map: every block as a card in the row of its layer, arrows to the blocks it uses. */
export default function BlockMap(): ReactNode {
  const [focus, setFocus] = useState<string | null>(null);
  const near = useMemo(() => {
    if (!focus) return null;
    const b = map.blocks.find((x) => x.name === focus);
    return new Set([focus, ...(b?.uses ?? []), ...(b?.usedBy ?? [])]);
  }, [focus]);
  const families = FAMILIES.filter((f) => map.blocks.some((b) => b.family === f.id));
  const pad = 14;
  const frame = useRef<HTMLDivElement>(null);
  const region = useScrollRegion(frame, 'Layer map');
  return (
    <figure className="as-map">
      <div className="as-map__frame" ref={frame} {...region}>
        <svg
          viewBox={`0 0 ${map.width} ${map.height}`}
          role="group"
          aria-label="Building blocks by layer, each linked to its page, with arrows to the blocks it depends on">
          <defs>
            <marker id="as-map-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
              <path className="as-map__arrow" d="M0 0L10 5L0 10z" />
            </marker>
            <marker id="as-map-arrow-on" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
              <path className="as-map__arrow--on" d="M0 0L10 5L0 10z" />
            </marker>
          </defs>
          {map.rows.map((r, i) => (
            <g key={r.layer}>
              {i % 2 === 0 && (
                <rect className="as-map__band" x={0} y={r.y - pad} width={map.width} height={r.h + 2 * pad} rx={10} />
              )}
              <text className="as-map__layer-n" x={10} y={r.y + 30}>
                {r.layer}
              </text>
              <text className="as-map__layer-name" x={10} y={r.y + 50}>
                {r.name}
              </text>
            </g>
          ))}
          {map.edges.map((e) => {
            const on = focus !== null && (e.from === focus || e.to === focus);
            const off = focus !== null && !on;
            return (
              <path
                key={`${e.from}-${e.to}`}
                className={`as-map__edge${on ? ' as-map__edge--on' : ''}${off ? ' as-map__edge--off' : ''}`}
                d={e.d}
                markerEnd={on ? 'url(#as-map-arrow-on)' : 'url(#as-map-arrow)'}>
                <title>{`${e.from} uses ${e.to} (${plural(e.count, 'reference')})`}</title>
              </path>
            );
          })}
          {map.blocks.map((b) => {
            const band = b.layer === 0;
            const state = near ? (near.has(b.name) ? (b.name === focus ? ' as-card--on' : '') : ' as-card--off') : '';
            return (
              <foreignObject key={b.name} x={b.x} y={b.y} width={b.w} height={b.h}>
                <Link
                  to={b.route}
                  className={`as-card as-fam-${b.family}${band ? ' as-card--band' : ''}${state}`}
                  title={b.summary}
                  onMouseEnter={() => setFocus(b.name)}
                  onMouseLeave={() => setFocus(null)}
                  onFocus={() => setFocus(b.name)}
                  onBlur={() => setFocus(null)}>
                  <span className="as-card__head">
                    <Icon block={b.name} className="as-card__icon" />
                    <span className="as-card__name">{b.name}</span>
                  </span>
                  <span className="as-card__summary">{band ? `${b.summary} Every block above uses it.` : b.summary}</span>
                  {!band && (
                    <span className="as-card__meta">
                      <span>
                        <b>{b.entities}</b> ent
                      </span>
                      <span>
                        <b>{b.commands}</b> cmd
                      </span>
                      <span>
                        <b>{b.events}</b> evt
                      </span>
                      <span className="as-card__kpi">
                        <b>{b.kpis}</b> KPI{b.kpis === 1 ? '' : 's'}
                      </span>
                    </span>
                  )}
                </Link>
              </foreignObject>
            );
          })}
        </svg>
      </div>
      <div className="as-map__legend" aria-label="Colour families">
        {families.map((f) => (
          <span key={f.id} className={`as-fam-${f.id}`}>
            <i />
            {f.label}
          </span>
        ))}
      </div>
      <p className="as-map__hint">
        Point at a block to see what it uses and what uses it. ent, cmd, evt: entities, commands and events of the block.
      </p>
    </figure>
  );
}
