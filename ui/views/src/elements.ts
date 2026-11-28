// The custom elements of @agenticspec/views, built from the same components as the Vue package:
// `<agentic-part-panel>`, `<agentic-overview>`, `<agentic-panel view=...>`, `<agentic-block-panels>`
// and one element per panel (`<agentic-harness-turns>`). Each takes `source`, `architecture`,
// `part` and `instance` as attributes. They draw in the light DOM, so the kit's stylesheet
// (agentic-views.css, linked by the host) and every `--kit-*` custom property the host sets on an
// ancestor apply unchanged. A page sends `agentic-navigate` and `agentic-command` DOM events.
import '@kit';
import { defineCustomElement, type Component } from 'vue';
import AgenticBlockPanels from './AgenticBlockPanels.vue';
import AgenticOverview from './AgenticOverview.vue';
import AgenticPanel from './AgenticPanel.vue';
import AgenticPartPage from './AgenticPartPage.vue';
import { PANELS, elementName } from './panels';
import { installViews } from './runtime';

function define(name: string, component: Component): void {
  if (customElements.get(name)) return;
  customElements.define(
    name,
    defineCustomElement(component as Parameters<typeof defineCustomElement>[0], {
      shadowRoot: false,
      configureApp: (app) => installViews(app),
    }),
  );
}

define('agentic-part-panel', AgenticPartPage);
define('agentic-overview', AgenticOverview);
define('agentic-panel', AgenticPanel);
define('agentic-block-panels', AgenticBlockPanels);
for (const [name, component] of Object.entries(PANELS)) define(elementName(name), component);
