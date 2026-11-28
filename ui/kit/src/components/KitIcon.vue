<script setup lang="ts">
// The icons of the kit, drawn in the current colour at 1em. Decorative: the control names itself.
export type IconName =
  | 'chevron-down'
  | 'chevron-right'
  | 'check'
  | 'copy'
  | 'search'
  | 'plus'
  | 'minus'
  | 'fit'
  | 'sun'
  | 'moon'
  | 'close'
  | 'alert'
  | 'refresh'
  | 'inbox'
  | 'menu';

defineProps<{ name: IconName }>();

const paths: Record<IconName, string> = {
  'chevron-down': 'M6 9l6 6 6-6',
  'chevron-right': 'M9 6l6 6-6 6',
  check: 'M5 12.5l4.5 4.5L19 7.5',
  copy: 'M9 9h10v10H9zM5 15V5h10',
  search: 'M11 18a7 7 0 1 1 0-14 7 7 0 0 1 0 14zM20 20l-4-4',
  plus: 'M12 5v14M5 12h14',
  minus: 'M5 12h14',
  fit: 'M4 9V4h5M20 9V4h-5M4 15v5h5M20 15v5h-5',
  sun: 'M12 16a4 4 0 1 0 0-8 4 4 0 0 0 0 8zM12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4',
  moon: 'M20 14.5A8 8 0 0 1 9.5 4a8 8 0 1 0 10.5 10.5z',
  close: 'M6 6l12 12M18 6L6 18',
  alert: 'M12 8v5M12 16.5v.5M10.3 3.9L2.4 18a2 2 0 0 0 1.7 3h15.8a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0z',
  refresh: 'M20 11a8 8 0 1 0-2.3 5.7M20 4v7h-7',
  inbox: 'M3 13h5l1.5 3h5l1.5-3h5M5 5h14l2 8v6H3v-6z',
  menu: 'M4 7h16M4 12h16M4 17h16',
};
</script>

<template>
  <svg class="kit-icon" viewBox="0 0 24 24" aria-hidden="true" focusable="false">
    <path :d="paths[name]" />
  </svg>
</template>

<style scoped>
.kit-icon {
  width: 1em;
  height: 1em;
  flex: none;
  fill: none;
  stroke: currentColor;
  stroke-width: 2;
  stroke-linecap: round;
  stroke-linejoin: round;
}
</style>
