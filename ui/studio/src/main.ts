// The studio app: the generated pages, given the host slots and the command hooks.
import '@kit';
import { createGeneratedApp } from './generated/app';
import { hooks } from './hooks';
import DraftGraph from './slots/DraftGraph.vue';
import PatchEditor from './slots/PatchEditor.vue';

createGeneratedApp({ hosts: { DraftGraph, PatchEditor }, hooks }).mount('#app');
