// The block panels: one component per view of the blocks, the widgets `agenticspec ui generate`
// writes from ui/views/blocks.ui.yaml. A panel file is named by its view in Pascal case
// (`agentic.harness.Turns` is `AgenticHarnessTurns`); every derived document copies the same file.
import type { Component } from 'vue';

const files = import.meta.glob<Component>('../gallery/src/generated/widgets/*.vue', { eager: true, import: 'default' });

/** Every panel, by its file name. */
export const PANELS: Record<string, Component> = Object.fromEntries(
  Object.entries(files).map(([path, c]) => [path.replace(/^.*\//, '').replace(/\.vue$/, ''), c]),
);

const pascal = (s: string) =>
  s
    .split('_')
    .map((w) => w.charAt(0).toUpperCase() + w.slice(1))
    .join('');

/** The file name of the panel of `view`: `agentic.harness.Turns` is `AgenticHarnessTurns`. */
export function panelName(view: string): string {
  return view.split('.').map(pascal).join('');
}

/** The panel of `view`, if the blocks have one. */
export function panelFor(view: string): Component | undefined {
  return PANELS[panelName(view)];
}

/** The custom element of a panel: `AgenticHarnessTurns` is `agentic-harness-turns`. */
export function elementName(panel: string): string {
  return panel.replace(/([a-z0-9])([A-Z])/g, '$1-$2').toLowerCase();
}

/**
 * The panels of an architecture block: those of the domain named like the block, else those of
 * the domain whose view lists the block's entity (`pool` is the `resource` domain's `Pools`).
 */
export function panelsOfBlock(block: string): string[] {
  const names = Object.keys(PANELS).sort();
  const of = (prefix: string) => names.filter((n) => n.startsWith(prefix) && /^[A-Z]/.test(n.slice(prefix.length)));
  const direct = of(`Agentic${pascal(block)}`);
  if (direct.length) return direct;
  const listing = names.find((n) => n.endsWith(`${pascal(block)}s`));
  return listing ? of(listing.slice(0, -`${pascal(block)}s`.length)) : [];
}
