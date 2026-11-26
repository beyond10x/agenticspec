// The pages of ui/agentspec.ui.yaml as paths. `agentspec serve` answers index.html for every
// path that is no file, so the paths are real URLs a host can open in a frame.

import { ref } from 'vue';

export type Route =
  | { page: 'overview' }
  | { page: 'session'; sessionId: string }
  | { page: 'embed.session'; sessionId: string }
  | { page: 'embed.timeline'; sessionId: string }
  | { page: 'embed.events'; sessionId: string };

export function parse(path: string): Route {
  const parts = path.split('/').filter(Boolean).map(decodeURIComponent);
  if (parts[0] === 'session' && parts[1]) return { page: 'session', sessionId: parts[1] };
  if (parts[0] === 'embed' && parts[2]) {
    if (parts[1] === 'session') return { page: 'embed.session', sessionId: parts[2] };
    if (parts[1] === 'timeline') return { page: 'embed.timeline', sessionId: parts[2] };
    if (parts[1] === 'events') return { page: 'embed.events', sessionId: parts[2] };
  }
  return { page: 'overview' };
}

export function href(route: Route): string {
  switch (route.page) {
    case 'overview':
      return '/';
    case 'session':
      return `/session/${encodeURIComponent(route.sessionId)}`;
    case 'embed.session':
      return `/embed/session/${encodeURIComponent(route.sessionId)}`;
    case 'embed.timeline':
      return `/embed/timeline/${encodeURIComponent(route.sessionId)}`;
    case 'embed.events':
      return `/embed/events/${encodeURIComponent(route.sessionId)}`;
  }
}

export const route = ref<Route>(typeof location === 'undefined' ? { page: 'overview' } : parse(location.pathname));

export function go(to: Route): void {
  const url = href(to);
  if (location.pathname !== url) history.pushState(null, '', url);
  route.value = to;
  window.scrollTo({ top: 0 });
}

if (typeof window !== 'undefined') {
  window.addEventListener('popstate', () => {
    route.value = parse(location.pathname);
  });
}
