// How the kit shows a value: one function per field kind, pure, so tables, lists and records
// show the same value the same way.

import type { FieldDef, Tone } from './types';

export function isEmpty(v: unknown): boolean {
  return v === null || v === undefined || v === '' || (Array.isArray(v) && v.length === 0);
}

/** The key of a value in a field's `labels` and `tones`. */
export function key(v: unknown): string {
  if (v === null || v === undefined) return '';
  return typeof v === 'object' ? JSON.stringify(v) : String(v);
}

/** A digest of 32 or more hex digits, which the kit shows short. */
export function isDigest(v: unknown): v is string {
  return typeof v === 'string' && /^(sha256:)?[0-9a-f]{32,}$/i.test(v);
}

/** The first 12 hex digits of a digest and an ellipsis. */
export function shortDigest(v: string): string {
  const hex = v.replace(/^sha256:/i, '');
  return hex.length > 12 ? `${hex.slice(0, 12)}…` : hex;
}

function pad(n: number): string {
  return String(n).padStart(2, '0');
}

function asDate(v: unknown): Date | null {
  if (typeof v !== 'string' && typeof v !== 'number') return null;
  const d = new Date(v);
  return Number.isNaN(d.getTime()) ? null : d;
}

/** A number with thin grouping, at most 3 decimals. */
export function formatNumber(v: unknown): string {
  const n = typeof v === 'number' ? v : Number(v);
  if (typeof v === 'boolean' || Number.isNaN(n)) return key(v);
  return new Intl.NumberFormat('en-US', { maximumFractionDigits: 3 }).format(n);
}

/** The text of one value as a field shows it (badges and lists take their own components). */
export function formatValue(def: Pick<FieldDef, 'as' | 'labels'>, v: unknown): string {
  if (isEmpty(v)) return '—';
  const label = def.labels?.[key(v)];
  if (label !== undefined) return label;
  switch (def.as) {
    case 'number':
      return formatNumber(v);
    case 'percent': {
      const n = Number(v);
      return Number.isNaN(n) ? key(v) : `${formatNumber(Math.round(n * 1000) / 10)} %`;
    }
    case 'toggle':
      return v === true || v === 'true' ? 'yes' : 'no';
    case 'timestamp': {
      const d = asDate(v);
      return d
        ? `${d.getUTCFullYear()}-${pad(d.getUTCMonth() + 1)}-${pad(d.getUTCDate())} ${pad(d.getUTCHours())}:${pad(d.getUTCMinutes())} UTC`
        : key(v);
    }
    case 'date': {
      const d = asDate(v);
      return d ? `${d.getUTCFullYear()}-${pad(d.getUTCMonth() + 1)}-${pad(d.getUTCDate())}` : key(v);
    }
    case 'time': {
      const d = asDate(v);
      return d ? `${pad(d.getUTCHours())}:${pad(d.getUTCMinutes())}:${pad(d.getUTCSeconds())}` : key(v);
    }
    case 'list':
      return Array.isArray(v) ? v.map(key).join(', ') : key(v);
    default:
      return isDigest(v) ? shortDigest(v) : key(v);
  }
}

/** The items of a list value. */
export function listItems(v: unknown): string[] {
  if (isEmpty(v)) return [];
  return Array.isArray(v) ? v.map(key) : [key(v)];
}

/** The tone of a badge value. */
export function toneOf(def: Pick<FieldDef, 'tones' | 'tone'>, v: unknown): Tone {
  return def.tones?.[key(v)] ?? def.tone ?? 'neutral';
}

/** A text long enough that a record shows it clamped to two lines. */
export function isLongText(v: unknown): boolean {
  return typeof v === 'string' && v.length > 90;
}
