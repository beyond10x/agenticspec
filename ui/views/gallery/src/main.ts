// The gallery of the block panels: the generated app over the fixtures, which answer a widget
// scoped to a part as the fixture source does. `?still` keeps the channel scripts from playing.
import '@kit';
import { createGeneratedApp } from './generated/app';
import { fixtureClient, scripts, source, views } from './fixtures';

const play = !new URLSearchParams(location.search).has('still');
createGeneratedApp({ hosts: {}, client: fixtureClient(views, scripts, { play, source }) }).mount('#app');
