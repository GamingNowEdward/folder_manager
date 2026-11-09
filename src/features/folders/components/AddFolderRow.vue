<script setup lang="ts">
const emit = defineEmits<{ click: [] }>()
</script>

<template>
  <div class="add-row" data-add-folder @click="emit('click')">
    <span class="add-row-icon">＋</span>
    <span class="add-row-text">添加文件夹</span>
  </div>
</template>

<style scoped>
.add-row {
  display: flex;
  align-items: center;
  gap: 10px;
  height: 38px;
  padding: 0 10px;
  border-radius: 8px;
  border: 1px dashed rgba(255, 255, 255, 0.12);
  color: rgba(255, 255, 255, 0.25);
  font-size: 12px;
  cursor: pointer;
  transition: all 0.15s;
}

.add-row:hover {
  border-color: rgba(96, 205, 255, 0.4);
  background: rgba(96, 205, 255, 0.04);
  color: rgba(255, 255, 255, 0.6);
}

.add-row-icon {
  font-size: 15px;
}
</style>
