// Which colour stands for what. A tool category keeps its colour on every page, whatever is
// filtered (colour follows the entity, never its rank).

import type { EventKind, Verdict } from './generated/types.ts';

export const TOOL_CATEGORIES: { id: string; label: string; color: string }[] = [
  { id: 'Execute', label: 'Shell', color: 'var(--s1)' },
  { id: 'Edit', label: 'Edit', color: 'var(--s2)' },
  { id: 'Read', label: 'Read', color: 'var(--s3)' },
  { id: 'Search', label: 'Search', color: 'var(--s4)' },
  { id: 'Message', label: 'Message', color: 'var(--s5)' },
  { id: 'Network', label: 'Network', color: 'var(--s6)' },
  { id: 'Spawn', label: 'Sub-agent', color: 'var(--s7)' },
  { id: 'Other', label: 'Other', color: 'var(--other)' },
];

export function toolColor(category: string | undefined): string {
  return TOOL_CATEGORIES.find((c) => c.id === category)?.color ?? 'var(--other)';
}

export function toolLabel(category: string | undefined): string {
  return TOOL_CATEGORIES.find((c) => c.id === category)?.label ?? 'Other';
}

export const KIND_LABELS: Record<EventKind, string> = {
  agent_started: 'Agent started',
  prompt: 'Prompt',
  model_turn: 'Model turn',
  tool_call: 'Tool call',
  decision: 'Decision',
  tool_result: 'Tool result',
  turn_end: 'Turn end',
  compaction: 'Compaction',
  exposure: 'Exposure',
  cache_bust: 'Cache bust',
};

export const KIND_PLURALS: Record<EventKind, string> = {
  agent_started: 'Agent starts',
  prompt: 'Prompts',
  model_turn: 'Model turns',
  tool_call: 'Tool calls',
  decision: 'Decisions',
  tool_result: 'Tool results',
  turn_end: 'Turn ends',
  compaction: 'Compactions',
  exposure: 'Exposures',
  cache_bust: 'Cache busts',
};

/** Who wrote a prompt, in words. */
export const AUTHOR: Record<string, string> = {
  Human: 'Human',
  Delegate: 'Operator session',
  PeerSession: 'Peer session',
  ParentAgent: 'Parent agent',
  Harness: 'Harness',
  Hook: 'Hook',
};

/** A check verdict as a status: colour, mark and words, never colour alone. */
export function verdictStatus(v: Verdict, severity: string): { color: string; mark: string; text: string } {
  if (v === 'Ok') return { color: 'var(--good)', mark: '✓', text: 'Passed' };
  if (v === 'Unk') return { color: 'var(--muted)', mark: '?', text: 'Unknown' };
  if (severity === 'high') return { color: 'var(--critical)', mark: '!', text: 'Found' };
  if (severity === 'low' || severity === 'medium') return { color: 'var(--serious)', mark: '!', text: 'Found' };
  return { color: 'var(--warning)', mark: '!', text: 'Found' };
}
