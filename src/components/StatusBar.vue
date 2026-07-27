<script setup lang="ts">
import { useAppStore } from '../stores/app'

const store = useAppStore()
</script>

<template>
  <div class="status-bar">
    {{ store.statusMessage }}
  </div>
</template>

<style scoped>
.status-bar {
  border-top: 1px solid rgba(255, 255, 255, 0.06);
  color: rgba(255, 255, 255, 0.3);
  font-size: 11px;
  padding: 5px 16px;
  flex-shrink: 0;
}
</style>
