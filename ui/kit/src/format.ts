// How the kit shows a value: one function per field kind, pure, so tables, lists and records
// show the same value the same way.

import type { FieldDef, Tone } from './types';

export function isEmpty(v: unknown): boolean {
  return v === null || v === undefined || v === '' || (Array.isArray(v) && v.length === 0);
}

/** A value that is a named object, such as a state with its summary: `{ name, ... }`. */
function named(v: unknown): string | null {
  if (v === null || typeof v !== 'object' || Array.isArray(v)) return null;
  const n = (v as Record<string, unknown>).name;
  return typeof n === 'string' ? n : null;
}

/** The key of a value in a field's `labels` and `tones`. */
export function key(v: unknown): string {
  if (v === null || v === undefined) return '';
  return typeof v === 'object' ? JSON.stringify(v) : String(v);
}

/** The key of a badge's value: a named object (a state with its summary) by its name. */
export function badgeKey(v: unknown): string {
  return named(v) ?? key(v);
}

/** The words a named object carries beside its name (its `summary`), for a title. */
export function summaryOf(v: unknown): string | undefined {
  if (named(v) === null) return undefined;
  const s = (v as Record<string, unknown>).summary;
  return typeof s === 'string' ? s : undefined;
}

/** A digest of 32 or more hex digits, which the kit shows short. */
export function isDigest(v: unknown): v is string {
  return typeof v === 'string' && /^(sha256:)?[0-9a-f]{32,}$/i.test(v);
}

/** A UUID (`8-4-4-4-12` hex digits), which the kit shows short. */
export function isUuid(v: unknown): v is string {
  return typeof v === 'string' && /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.test(v);
}

/** An ellipsis and the last 12 hex digits of a UUID, the part that tells two apart. */
export function shortUuid(v: string): string {
  return `…${v.slice(-12)}`;
}

/** The entries of a JSON value as `key: value` lines: an object's fields, a list's items, or the value. */
export function jsonEntries(v: unknown): string[] {
  if (isEmpty(v)) return [];
  if (Array.isArray(v)) return v.map((x) => (typeof x === 'object' && x !== null ? JSON.stringify(x) : key(x)));
  if (typeof v === 'object' && v !== null) {
    return Object.entries(v as Record<string, unknown>).map(([k, x]) => `${k}: ${typeof x === 'object' && x !== null ? JSON.stringify(x) : key(x)}`);
  }
  return [key(v)];
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
  const k = def.as === 'badge' ? badgeKey(v) : key(v);
  const label = def.labels?.[k];
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
    case 'badge':
      return k;
    case 'json':
      return jsonEntries(v).join(', ');
    default:
      return isDigest(v) ? shortDigest(v) : isUuid(v) ? shortUuid(v) : key(v);
  }
}

/** The items of a list value. */
export function listItems(v: unknown): string[] {
  if (isEmpty(v)) return [];
  return Array.isArray(v) ? v.map(key) : [key(v)];
}

/** The tone of a badge value. */
export function toneOf(def: Pick<FieldDef, 'tones' | 'tone'>, v: unknown): Tone {
  return def.tones?.[badgeKey(v)] ?? def.tone ?? 'neutral';
}

/** A text long enough that a record shows it clamped to two lines. */
export function isLongText(v: unknown): boolean {
  return typeof v === 'string' && v.length > 90;
}
