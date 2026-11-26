// Light, dark, or the system's choice; the choice is kept in localStorage.

import { ref, watch } from 'vue';

export type Theme = 'system' | 'light' | 'dark';

const KEY = 'sessionspec.theme';

function initial(): Theme {
  if (typeof window === 'undefined') return 'system';
  const fromUrl = new URLSearchParams(location.search).get('theme');
  if (fromUrl === 'light' || fromUrl === 'dark') return fromUrl;
  const saved = localStorage.getItem(KEY);
  return saved === 'light' || saved === 'dark' ? saved : 'system';
}

export const theme = ref<Theme>(initial());

function apply(t: Theme) {
  if (typeof document === 'undefined') return;
  if (t === 'system') document.documentElement.removeAttribute('data-theme');
  else document.documentElement.setAttribute('data-theme', t);
}

apply(theme.value);
watch(theme, (t) => {
  apply(t);
  if (typeof localStorage !== 'undefined') {
    if (t === 'system') localStorage.removeItem(KEY);
    else localStorage.setItem(KEY, t);
  }
});

export function isDark(): boolean {
  if (theme.value !== 'system') return theme.value === 'dark';
  return typeof matchMedia !== 'undefined' && matchMedia('(prefers-color-scheme: dark)').matches;
}

export function toggleTheme(): void {
  theme.value = isDark() ? 'light' : 'dark';
}
