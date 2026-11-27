/**
 * The architecture canvas of the ladder: every part at its fixed grid cell, the links between
 * them, and a camera that frames the parts of the current rung. Parts that arrive grow in, links
 * that arrive draw themselves, parts that go fade out; the owner stays at the left edge.
 */
import {useMemo, type ReactNode} from 'react';
import clsx from 'clsx';
import {iconPath} from '@site/src/components/blocks/families';
import {ladder, type Part, type Rung} from './data';

export interface Geometry {
  cw: number;
  ch: number;
  nw: number;
  nh: number;
  padL: number;
  padT: number;
  padR: number;
  padB: number;
  transposed: boolean;
  maxScale: number;
}

export const WIDE: Geometry = {
  cw: 166,
  ch: 86,
  nw: 148,
  nh: 54,
  padL: 34,
  padT: 14,
  padR: 14,
  padB: 14,
  transposed: false,
  maxScale: 1.45,
};

export const NARROW: Geometry = {
  cw: 70,
  ch: 60,
  nw: 64,
  nh: 48,
  padL: 6,
  padT: 26,
  padR: 6,
  padB: 6,
  transposed: true,
  maxScale: 1.3,
};

const EXTRA_ICONS: Record<string, string> = {
  trigger: 'M8 1.5L3 8h3.5L5.5 12.5 11 6H7.5z',
  schedule: 'M7 12.5a5.5 5.5 0 1 0 0-11 5.5 5.5 0 0 0 0 11zM7 4v3.2l2.2 1.4',
  experiment:
    'M5.5 1.8h3M6 1.8v3.6L2.6 11a1 1 0 0 0 .9 1.4h7a1 1 0 0 0 .9-1.4L8 5.4V1.8M4.2 8.6h5.6',
};

const SHIELD = 'M0 -6.2l5 2v3.4c0 2.9-2.1 4.6-5 5.4-2.9-.8-5-2.5-5-5.4v-3.4z';
const CHECK = 'M-2.3 0.1l1.6 1.6 3.1-3.3';

function icon(block: string): string {
  return EXTRA_ICONS[block] ?? iconPath(block);
}

function origin(g: Geometry, id: string): {x: number; y: number} {
  const at = ladder.layout.parts[id] ?? {col: 0, row: 0};
  return g.transposed
    ? {x: g.padL + at.row * g.cw, y: g.padT + at.col * g.ch}
    : {x: g.padL + at.col * g.cw, y: g.padT + at.row * g.ch};
}

export function size(g: Geometry): {w: number; h: number} {
  const along = ladder.layout.cols - 1;
  const across = ladder.layout.rows - 1;
  return g.transposed
    ? {w: g.padL + across * g.cw + g.nw + g.padR, h: g.padT + along * g.ch + g.nh + g.padB}
    : {w: g.padL + along * g.cw + g.nw + g.padR, h: g.padT + across * g.ch + g.nh + g.padB};
}

interface Camera {
  transform: string;
  /** The right (wide) or bottom (narrow) edge of what the rung uses, in canvas units. */
  x1: number;
  y1: number;
  /** The height of the canvas: fixed when wide, the rung's own height when narrow. */
  height: number;
}

/**
 * The camera that frames rung `r`. Wide: the whole canvas keeps its size and the rung is zoomed
 * and centred in it. Narrow: the rung fills the width and the canvas is as tall as the rung.
 */
function camera(g: Geometry, r: Rung): Camera {
  const {w, h} = size(g);
  const v = r.view;
  const corner = (col: number, row: number) =>
    g.transposed
      ? {x: g.padL + row * g.cw, y: g.padT + col * g.ch}
      : {x: g.padL + col * g.cw, y: g.padT + row * g.ch};
  const p0 = corner(v.col0, v.row0);
  const p1 = corner(v.col1, v.row1);
  const margin = 12;
  const x0 = g.transposed ? Math.max(0, Math.min(p0.x, p1.x) - margin) : 0;
  const y0 = g.transposed ? 0 : Math.min(p0.y, p1.y) - margin;
  const x1 = Math.max(p0.x, p1.x) + g.nw + margin;
  const y1 = Math.max(p0.y, p1.y) + g.nh + margin;
  const bw = Math.max(1, x1 - x0);
  const bh = Math.max(1, y1 - y0);
  const s = g.transposed ? Math.min(w / bw, g.maxScale) : Math.min(w / bw, h / bh, g.maxScale);
  const tx = (w - bw * s) / 2 - x0 * s;
  const ty = g.transposed ? 0 : (h - bh * s) / 2 - y0 * s;
  return {
    transform: `translate(${tx.toFixed(1)}px, ${ty.toFixed(1)}px) scale(${s.toFixed(4)})`,
    x1,
    y1,
    height: g.transposed ? Math.ceil(bh * s) : h,
  };
}

interface Edge {
  key: string;
  a: string;
  b: string;
  carries: string[];
}

function edges(r: Rung | undefined): Map<string, Edge> {
  const out = new Map<string, Edge>();
  for (const l of r?.links ?? []) {
    const [a, b] = l.from < l.to ? [l.from, l.to] : [l.to, l.from];
    const key = `${a}|${b}`;
    const e = out.get(key) ?? {key, a, b, carries: []};
    if (!e.carries.includes(l.carries)) e.carries.push(l.carries);
    out.set(key, e);
  }
  return out;
}

function addedPairs(r: Rung): Set<string> {
  return new Set(
    r.added_links.map((k) => {
      const m = /^(.*)>(.*):(.*)$/.exec(k);
      if (!m) return k;
      return m[1] < m[2] ? `${m[1]}|${m[2]}` : `${m[2]}|${m[1]}`;
    }),
  );
}

/** Where the segment from the centre of a box towards (tx, ty) leaves the box. */
function exit(cx: number, cy: number, hw: number, hh: number, tx: number, ty: number) {
  const dx = tx - cx;
  const dy = ty - cy;
  if (dx === 0 && dy === 0) return {x: cx, y: cy};
  const s = Math.min(dx === 0 ? Infinity : hw / Math.abs(dx), dy === 0 ? Infinity : hh / Math.abs(dy));
  return {x: cx + dx * s, y: cy + dy * s};
}

function path(g: Geometry, e: Edge): string {
  const pa = origin(g, e.a);
  const pb = origin(g, e.b);
  const ca = {x: pa.x + g.nw / 2, y: pa.y + g.nh / 2};
  const cb = {x: pb.x + g.nw / 2, y: pb.y + g.nh / 2};
  const len = Math.hypot(cb.x - ca.x, cb.y - ca.y) || 1;
  const bend = Math.min(30, len * 0.1) * ((e.a.length + e.b.length) % 2 === 0 ? 1 : -1);
  const mx = (ca.x + cb.x) / 2 + (-(cb.y - ca.y) / len) * bend;
  const my = (ca.y + cb.y) / 2 + ((cb.x - ca.x) / len) * bend;
  const s = exit(ca.x, ca.y, g.nw / 2 + 3, g.nh / 2 + 3, mx, my);
  const t = exit(cb.x, cb.y, g.nw / 2 + 3, g.nh / 2 + 3, mx, my);
  return `M${s.x.toFixed(1)} ${s.y.toFixed(1)}Q${mx.toFixed(1)} ${my.toFixed(1)} ${t.x.toFixed(1)} ${t.y.toFixed(1)}`;
}

function Node({
  g,
  part,
  state,
}: {
  g: Geometry;
  part: Part;
  state: {added: boolean; changed: boolean; entering: boolean; leaving: boolean};
}): ReactNode {
  const {x, y} = origin(g, part.id);
  const narrow = g.transposed;
  return (
    <g
      className={clsx(
        'ld-node',
        `as-fam-${part.family}`,
        part.trust && 'ld-node--trust',
        state.added && 'ld-node--added',
        state.changed && 'ld-node--changed',
        state.entering && 'ld-node--entering',
        state.leaving && 'ld-node--leaving',
      )}
      style={{transform: `translate(${x}px, ${y}px)`}}
      aria-hidden={state.leaving || undefined}>
      <g className="ld-node__body">
        {part.count > 1 && (
          <>
            <rect className="ld-node__stack" x={7} y={-7} width={g.nw} height={g.nh} rx={10} />
            <rect className="ld-node__stack" x={3.5} y={-3.5} width={g.nw} height={g.nh} rx={10} />
          </>
        )}
        <rect className="ld-node__card" width={g.nw} height={g.nh} rx={narrow ? 8 : 10} />
        {narrow ? (
          <rect className="ld-node__bar" x={0} y={0} width={g.nw} height={4} rx={2} />
        ) : (
          <rect className="ld-node__bar" x={0} y={0} width={4} height={g.nh} rx={2} />
        )}
        <svg
          className="ld-node__icon"
          x={narrow ? g.nw / 2 - 7 : 12}
          y={narrow ? 9 : 10}
          width={14}
          height={14}
          viewBox="0 0 14 14">
          <path d={icon(part.block)} />
        </svg>
        <text
          className="ld-node__name"
          x={narrow ? g.nw / 2 : 32}
          y={narrow ? g.nh - 11 : 22}
          textAnchor={narrow ? 'middle' : 'start'}>
          {part.id}
        </text>
        {!narrow && (
          <text className="ld-node__label" x={12} y={42}>
            {part.label}
          </text>
        )}
        {part.count > 1 && (
          <g className="ld-node__count" transform={`translate(${g.nw - (narrow ? 2 : 4)}, ${narrow ? -2 : -9})`}>
            <rect x={narrow ? -22 : -30} y={narrow ? -8 : 0} width={narrow ? 26 : 30} height={narrow ? 16 : 18} rx={9} />
            <text x={narrow ? -9 : -15} y={narrow ? 4 : 13} textAnchor="middle">
              ×{part.count}
            </text>
          </g>
        )}
        {part.trust && (
          <g className="ld-shield" transform={narrow ? 'translate(4, 4)' : 'translate(0, 0)'}>
            <circle r={narrow ? 9 : 11} />
            <g transform={narrow ? 'scale(0.82)' : undefined}>
              <path className="ld-shield__body" d={SHIELD} />
              <path className="ld-shield__check" d={CHECK} />
            </g>
          </g>
        )}
      </g>
    </g>
  );
}

export default function Canvas({
  g,
  rung,
  previous,
  dim,
  title,
}: {
  g: Geometry;
  rung: Rung;
  previous: Rung | undefined;
  dim: boolean;
  title: string;
}): ReactNode {
  const {w} = size(g);
  const now = useMemo(() => edges(rung), [rung]);
  const before = useMemo(() => edges(previous), [previous]);
  const added = useMemo(() => addedPairs(rung), [rung]);
  const present = new Set(rung.parts.map((p) => p.id));
  const had = new Set((previous?.parts ?? []).map((p) => p.id));
  const leavingParts = (previous?.parts ?? []).filter((p) => !present.has(p.id));
  const leavingEdges = [...before.values()].filter((e) => !now.has(e.key));
  const moved = previous !== undefined && previous !== rung;
  const bands = ladder.layout.bands;
  const cam = camera(g, rung);
  const v = rung.view;

  const band = (b0: number, b1: number) => {
    const row0 = Math.max(b0, Math.min(b1, v.row0));
    const row1 = Math.max(row0, Math.min(b1, v.row1));
    if (g.transposed) {
      return {x: g.padL + row0 * g.cw - 3, y: 0, w: (row1 - row0) * g.cw + g.nw + 6, h: cam.y1};
    }
    return {x: 0, y: g.padT + row0 * g.ch - 6, w: cam.x1, h: (row1 - row0) * g.ch + g.nh + 12};
  };

  return (
    <svg
      className={clsx('ld-canvas', dim && 'ld-canvas--dim')}
      viewBox={`0 0 ${w} ${cam.height}`}
      role="img"
      aria-label={title}>
      <g className="ld-camera" style={{transform: cam.transform}}>
        {bands.map((b) => {
          const r = band(b.row0, b.row1);
          const trust = b.name === 'trust';
          const out = b.row0 > v.row1 || b.row1 < v.row0;
          return (
            <g key={b.name} className={clsx('ld-band', trust && 'ld-band--trust', out && 'ld-band--out')}>
              <rect x={r.x} y={r.y} rx={12} style={{width: r.w, height: r.h}} />
              {g.transposed ? (
                <text className="ld-band__name" x={r.x + r.w / 2} y={15} textAnchor="middle">
                  {trust ? 'trust' : b.name === 'works' ? 'works' : 'feeds'}
                </text>
              ) : (
                <text
                  className="ld-band__name"
                  transform={`translate(${17}, ${r.y + r.h / 2}) rotate(-90)`}
                  textAnchor="middle">
                  {trust ? 'trust' : b.name === 'works' ? 'works' : 'feeds'}
                </text>
              )}
            </g>
          );
        })}
        <g className="ld-edges">
          {leavingEdges.map((e) => (
            <path key={`x-${e.key}`} className="ld-edge ld-edge--leaving" d={path(g, e)} />
          ))}
          {[...now.values()].map((e) => (
            <path
              key={e.key}
              className={clsx(
                'ld-edge',
                added.has(e.key) && 'ld-edge--added',
                moved && !before.has(e.key) && 'ld-edge--entering',
              )}
              d={path(g, e)}
              pathLength={1}
            />
          ))}
        </g>
        <g className="ld-nodes">
          {leavingParts.map((p) => (
            <Node
              key={p.id}
              g={g}
              part={p}
              state={{added: false, changed: false, entering: false, leaving: true}}
            />
          ))}
          {rung.parts.map((p) => (
            <Node
              key={p.id}
              g={g}
              part={p}
              state={{
                added: rung.added_parts.includes(p.id),
                changed: rung.changed_parts.includes(p.id),
                entering: moved && !had.has(p.id),
                leaving: false,
              }}
            />
          ))}
        </g>
      </g>
    </svg>
  );
}
