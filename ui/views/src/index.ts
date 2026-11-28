// @agenticspec/views: the block panels, the part pages and the overviews of a derived UI, as Vue
// components over ui/kit. A Vue app installs the panels' runtime once (`installViews`, or
// `mountViews` for an island in an app that has a runtime of its own) and places the components;
// each is parameterised by `source`, `architecture`, `part` and `instance` only. A page sends
// `agentic-navigate` and `agentic-command` DOM events instead of acting itself.
//
// `source` is the base URL of a server that answers `GET <source>/api/views/<view>`,
// `GET <source>/api/channels/<channel>` and `GET <source>/layouts/<architecture>.json`. The
// client sends the browser's cookies, so a server on another origin answers CORS with
// `Access-Control-Allow-Credentials: true` and the host page's origin (not `*`).
import { createApp, h, type Component } from 'vue';
import { installViews, type ViewsOptions } from './runtime';

export { default as AgenticPanel } from './AgenticPanel.vue';
export { default as AgenticPartPage } from './AgenticPartPage.vue';
export { default as AgenticOverview } from './AgenticOverview.vue';
export { default as AgenticBlockPanels } from './AgenticBlockPanels.vue';
export { PANELS, panelFor, panelName, panelsOfBlock, elementName } from './panels';
export { loadLayout, pageOf } from './layout';
export type { Layout, LayoutLevel, LayoutPage, LayoutPart, LayoutFrame, LayoutSection, LayoutAction } from './layout';
export type { NavigateDetail, CommandDetail } from './events';
export * from './runtime';

/**
 * Mounts `component` with `props` into `el` as an app of its own with the panels' runtime
 * installed: for a host app whose own runtime is another (agenticstudio). Returns the unmount.
 */
export function mountViews(el: Element, component: Component, props: Record<string, unknown>, options: ViewsOptions = {}): () => void {
  const app = createApp({ render: () => h(component, props) });
  installViews(app, options);
  app.mount(el);
  return () => app.unmount();
}
