/**
 * Colour families and icons of the building blocks. The eight families of parts use the colours
 * and the 14 by 14 stroke icons of the architecture renderer, so a block reads the same on the
 * layer map, on its page and in a rendered architecture.
 */

export const FAMILIES: {id: string; label: string}[] = [
  {id: 'principal', label: 'principals'},
  {id: 'intel', label: 'intelligence'},
  {id: 'capability', label: 'capability'},
  {id: 'agent', label: 'agent'},
  {id: 'work', label: 'work and governance'},
  {id: 'collab', label: 'collaboration'},
  {id: 'place', label: 'place'},
  {id: 'resource', label: 'resource'},
  {id: 'composition', label: 'composition'},
  {id: 'evaluation', label: 'evaluation'},
  {id: 'foundation', label: 'foundation'},
];

const ICONS: Record<string, string> = {
  principal: 'M7 6.6a2.4 2.4 0 1 0 0-4.8 2.4 2.4 0 0 0 0 4.8zM2.4 12.6c0-2.6 2-4.2 4.6-4.2s4.6 1.6 4.6 4.2',
  delegation: 'M7 6.6a2.4 2.4 0 1 0 0-4.8 2.4 2.4 0 0 0 0 4.8zM2.4 12.6c0-2.6 2-4.2 4.6-4.2s4.6 1.6 4.6 4.2',
  agent:
    'M3.5 4.5h7a1.5 1.5 0 0 1 1.5 1.5v4.5a1.5 1.5 0 0 1-1.5 1.5h-7A1.5 1.5 0 0 1 2 10.5V6a1.5 1.5 0 0 1 1.5-1.5zM7 4.5V2M5.2 8.2h.01M8.8 8.2h.01',
  harness:
    'M3.5 4.5h7a1.5 1.5 0 0 1 1.5 1.5v4.5a1.5 1.5 0 0 1-1.5 1.5h-7A1.5 1.5 0 0 1 2 10.5V6a1.5 1.5 0 0 1 1.5-1.5zM7 4.5V2M5.2 8.2h.01M8.8 8.2h.01',
  channel: 'M2 2.5h10v7H6.5L3.5 12V9.5H2z',
  question: 'M2 2.5h10v7H6.5L3.5 12V9.5H2z',
  gateway: 'M2 2.5v9M5 7h7M9 4l3 3-3 3',
  model:
    'M4 4h6v6H4zM5.5 1.5V4M8.5 1.5V4M5.5 10v2.5M8.5 10v2.5M1.5 5.5H4M1.5 8.5H4M10 5.5h2.5M10 8.5h2.5',
  tool: 'M9.6 1.8a3 3 0 0 0-3.4 4.1L2 10.1 3.9 12l4.2-4.2a3 3 0 0 0 4.1-3.4L10.4 6.2 8.6 5.4 7.8 3.6z',
  knowledge: 'M9.6 1.8a3 3 0 0 0-3.4 4.1L2 10.1 3.9 12l4.2-4.2a3 3 0 0 0 4.1-3.4L10.4 6.2 8.6 5.4 7.8 3.6z',
  workspace: 'M1.5 3.5h4l1.5 1.5h5.5v6.5h-11z',
  sandbox: 'M7 1.5l5 2v3.6c0 2.9-2.1 4.7-5 5.4-2.9-.7-5-2.5-5-5.4V3.5z',
  runtime: 'M4 2.5v9l7.5-4.5z',
  resource: 'M2 2.5h10v2.5H2zM2 6.25h10v2.5H2zM2 10h10v2H2z',
  work: 'M2 3.8l1.4 1.4L6 2.6M2 9l1.4 1.4L6 7.8M8 4.2h4.5M8 9.4h4.5',
  process: 'M2 3.8l1.4 1.4L6 2.6M2 9l1.4 1.4L6 7.8M8 4.2h4.5M8 9.4h4.5',
  authority: 'M3.5 6.5h7v5.5h-7zM5 6.5V4.8a2 2 0 0 1 4 0v1.7',
  shared: 'M7 1.8l5.2 2.7L7 7.2 1.8 4.5zM1.8 7l5.2 2.7L12.2 7M1.8 9.5l5.2 2.7 5.2-2.7',
  architecture: 'M3 3h2.5v2.5H3zM8.5 3H11v2.5H8.5zM5.75 8.5h2.5V11h-2.5zM5.5 4.25h3M4.25 5.5l2 3M9.75 5.5l-2 3',
  kpi: 'M2 12h10M3.5 12V8M6 12V4.5M8.5 12V6.5M11 12V2.5',
  experiment: 'M5.5 1.8h3M6 1.8v3.6L2.6 11a1 1 0 0 0 .9 1.4h7a1 1 0 0 0 .9-1.4L8 5.4V1.8M4.2 8.6h5.6',
};

const FALLBACK = 'M7 12A5 5 0 1 0 7 2a5 5 0 0 0 0 10z';

export function iconPath(block: string): string {
  return ICONS[block] ?? FALLBACK;
}
