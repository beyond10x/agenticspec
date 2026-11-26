// How numbers, times and names read in the UI. Pure functions, tested in format.test.ts.

export function money(usd: number | null | undefined): string {
  if (usd === null || usd === undefined || Number.isNaN(usd)) return '–';
  if (usd === 0) return '$0';
  if (Math.abs(usd) < 0.01) return '<$0.01';
  if (Math.abs(usd) < 100) return `$${usd.toFixed(2)}`;
  if (Math.abs(usd) < 10_000) return `$${Math.round(usd).toLocaleString('en-US')}`;
  return `$${compact(usd)}`;
}

export function compact(n: number | null | undefined): string {
  if (n === null || n === undefined || Number.isNaN(n)) return '–';
  const a = Math.abs(n);
  if (a < 1_000) return `${Math.round(n)}`;
  if (a < 1_000_000) return `${trim(n / 1_000)}K`;
  if (a < 1_000_000_000) return `${trim(n / 1_000_000)}M`;
  return `${trim(n / 1_000_000_000)}B`;
}

function trim(x: number): string {
  return Math.abs(x) >= 100 ? `${Math.round(x)}` : x.toFixed(1).replace(/\.0$/, '');
}

export function int(n: number | null | undefined): string {
  if (n === null || n === undefined || Number.isNaN(n)) return '–';
  return Math.round(n).toLocaleString('en-US');
}

export function percent(share: number | null | undefined, digits = 0): string {
  if (share === null || share === undefined || Number.isNaN(share)) return '–';
  const p = share * 100;
  if (p > 0 && p < 1 && digits === 0) return '<1%';
  return `${p.toFixed(digits)}%`;
}

/** A duration in seconds: `45 s`, `13 min`, `2 h 13 min`, `3 d 4 h`. */
export function duration(seconds: number | null | undefined): string {
  if (seconds === null || seconds === undefined || Number.isNaN(seconds)) return '–';
  const s = Math.max(0, Math.round(seconds));
  if (s < 60) return `${s} s`;
  const m = Math.round(s / 60);
  if (m < 60) return `${m} min`;
  const h = Math.floor(m / 60);
  const rm = m % 60;
  if (h < 24) return rm ? `${h} h ${rm} min` : `${h} h`;
  const d = Math.floor(h / 24);
  const rh = h % 24;
  return rh ? `${d} d ${rh} h` : `${d} d`;
}

/** A duration in milliseconds, compact for axis and gap labels: `45s`, `13m`, `2h 13m`. */
export function shortDuration(ms: number): string {
  const s = Math.max(0, Math.round(ms / 1000));
  if (s < 60) return `${s}s`;
  const m = Math.round(s / 60);
  if (m < 60) return `${m}m`;
  const h = Math.floor(m / 60);
  const rm = m % 60;
  if (h < 24) return rm ? `${h}h ${rm}m` : `${h}h`;
  const d = Math.floor(h / 24);
  return `${d}d ${h % 24}h`;
}

/** How long ago, from `now` (ms): `just now`, `12 min ago`, `3 h ago`, `yesterday`, `4 d ago`. */
export function ago(at: number, now: number): string {
  const s = Math.max(0, (now - at) / 1000);
  if (s < 45) return 'just now';
  if (s < 3600) return `${Math.max(1, Math.round(s / 60))} min ago`;
  if (s < 86_400) return `${Math.round(s / 3600)} h ago`;
  if (s < 2 * 86_400) return 'yesterday';
  return `${Math.round(s / 86_400)} d ago`;
}

export function clock(at: number): string {
  const d = new Date(at);
  return `${pad(d.getHours())}:${pad(d.getMinutes())}`;
}

export function clockSeconds(at: number): string {
  const d = new Date(at);
  return `${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`;
}

const MONTHS = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];

export function day(at: number): string {
  const d = new Date(at);
  return `${MONTHS[d.getMonth()]} ${d.getDate()}`;
}

export function dayClock(at: number): string {
  return `${day(at)}, ${clock(at)}`;
}

function pad(n: number): string {
  return n < 10 ? `0${n}` : `${n}`;
}

/** A model id without the vendor prefix and date suffix: `claude-opus-5-5` -> `opus 5.5`. */
export function modelName(model: string | null | undefined): string {
  if (!model) return 'unknown model';
  const m = /^claude-([a-z]+)-(\d+)(?:-(\d+))?/.exec(model);
  if (m) return m[3] ? `${m[1]} ${m[2]}.${m[3]}` : `${m[1]} ${m[2]}`;
  return model;
}

/** The last path segments of a home-relative project, for a narrow column. */
export function projectName(project: string): string {
  const parts = project.replace(/^~\/?/, '').split('/').filter(Boolean);
  if (parts.length === 0) return '~';
  return parts.slice(-2).join('/');
}

export function plural(n: number, one: string, many = `${one}s`): string {
  return `${int(n)} ${n === 1 ? one : many}`;
}
