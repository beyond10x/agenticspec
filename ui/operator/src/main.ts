// The operator app `agenticspec ui build` writes: one build for every architecture. It reads its
// data files, installs the panels' runtime with a client of the runtime interface (R1-R5) at the
// adapter's base path, and draws the overviews and part pages of the layout with the kit.
import '@kit';
import { createApp, h } from 'vue';
import { createClient, installViews } from '../../views/src/index';
import App from './App.vue';
import { apiRoot, loadData } from './data';

const el = document.getElementById('app');
loadData().then(
  (data) => {
    document.title = data.config.title;
    const client = createClient(apiRoot(data.config));
    const app = createApp({ render: () => h(App, { data, client }) });
    installViews(app, { client });
    app.mount('#app');
  },
  (e: unknown) => {
    if (el) el.textContent = `The operator UI could not read its data files: ${e instanceof Error ? e.message : String(e)}`;
  },
);
