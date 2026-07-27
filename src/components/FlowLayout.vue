<script setup lang="ts">
</script>

<template>
  <div class="flow-layout">
    <slot />
  </div>
</template>

<style scoped>
.flow-layout {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  align-content: flex-start;
}
</style>
