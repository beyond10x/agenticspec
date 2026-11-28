// A React page that places the custom elements of @agenticspec/views: it loads the built bundle
// and its stylesheet as any other page would, and keeps its own state of the last event.
import { createElement as h, useEffect, useRef, useState } from 'react';
import { createRoot } from 'react-dom/client';

for (const [tag, attrs] of [
  ['link', { rel: 'stylesheet', href: '/elements/agentic-views.css' }],
  ['script', { type: 'module', src: '/elements/agentic-views.js' }],
]) {
  document.head.append(Object.assign(document.createElement(tag), attrs));
}

function App() {
  const host = useRef(null);
  const [last, setLast] = useState('no event yet');
  useEffect(() => {
    const el = host.current;
    const on = (e) => setLast(`${e.type} ${JSON.stringify(e.detail)}`);
    el.addEventListener('agentic-navigate', on);
    el.addEventListener('agentic-command', on);
    return () => {
      el.removeEventListener('agentic-navigate', on);
      el.removeEventListener('agentic-command', on);
    };
  }, []);
  return h(
    'main',
    null,
    h('h1', null, 'A React page'),
    h('p', { id: 'last' }, last),
    h(
      'div',
      { ref: host, style: { '--kit-accent': 'rgb(200, 30, 120)' } },
      h('agentic-overview', { source: '/fixtures', architecture: 'workstation' }),
      h('agentic-part-panel', { source: '/fixtures', architecture: 'workstation', part: 'operator' }),
    ),
  );
}

createRoot(document.getElementById('root')).render(h(App));
