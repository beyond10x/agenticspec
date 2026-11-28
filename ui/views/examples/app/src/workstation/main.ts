// The UI derived from architectures/workstation.yaml and its contract, over the fixtures. `?still`
// keeps the channel's script from playing.
import '@kit';
import { createGeneratedApp } from './generated/app';
import { Refused, type Client } from './generated/client';
import { exampleClient } from '../fixtures';
import ArchitectureSettings from '../ArchitectureSettings.vue';
import DraftGraph from '../../../../../studio/src/slots/DraftGraph.vue';

const play = !new URLSearchParams(location.search).has('still');
const client = exampleClient('workstation', { play, refuse: (s, c, m) => new Refused(s, c, m) });
createGeneratedApp({ hosts: { ArchitectureSettings, DraftGraph }, client: client as unknown as Client }).mount('#app');
