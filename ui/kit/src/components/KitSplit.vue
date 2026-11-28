<script setup lang="ts">
// Two panes: the main one takes the width and the full height, the side one is narrow and
// scrolls on its own. Below 900 px they stack, the main one first.
</script>

<template>
  <div class="kit-split">
    <div class="main-pane">
      <slot name="main" />
    </div>
    <aside class="side-pane">
      <slot name="side" />
    </aside>
  </div>
</template>

<style scoped>
.kit-split {
  display: grid;
  grid-template-columns: minmax(0, 1fr) var(--kit-side-width);
  height: 100%;
  min-height: 0;
}
.main-pane {
  display: flex;
  flex-direction: column;
  min-width: 0;
  min-height: 0;
}
.side-pane {
  min-width: 0;
  min-height: 0;
  overflow-y: auto;
  background: var(--kit-surface-muted);
  border-left: 1px solid var(--kit-hairline);
}
.side-pane > :deep(.kit-stack.padded) {
  padding: var(--kit-space-3);
  gap: var(--kit-space-2);
}
@media (max-width: 900px) {
  .kit-split {
    grid-template-columns: minmax(0, 1fr);
    grid-template-rows: minmax(22rem, 70vh) auto;
    height: auto;
  }
  .side-pane {
    overflow: visible;
    border-left: 0;
    border-top: 1px solid var(--kit-hairline);
  }
}
</style>
