// The gallery of the block panels: the generated app over the fixtures. `?still` keeps the
// channel scripts from playing.
import '@kit';
import { createGeneratedApp } from './generated/app';
import { fixtureClient, scripts, views } from './fixtures';

const play = !new URLSearchParams(location.search).has('still');
createGeneratedApp({ hosts: {}, client: fixtureClient(views, scripts, { play }) }).mount('#app');
