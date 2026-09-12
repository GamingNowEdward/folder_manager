<script setup lang="ts">
import { useStatusStore } from '@/shared/stores/status.store'

const statusStore = useStatusStore()
</script>

<template>
  <div class="status-bar">
    {{ statusStore.message }}
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
