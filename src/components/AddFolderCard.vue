<script setup lang="ts">
const emit = defineEmits<{ click: [] }>()
</script>

<template>
  <div class="add-card" @click="emit('click')">
    <div class="add-icon">＋</div>
    <div class="add-text">添加文件夹</div>
  </div>
</template>

<style scoped>
.add-card {
  background: rgba(255, 255, 255, 0.02);
  border: 1.5px dashed rgba(255, 255, 255, 0.15);
  border-radius: 14px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 6px;
  min-height: 100px;
  cursor: pointer;
  transition: all 0.15s;
}

.add-card:hover {
  border-color: rgba(96, 205, 255, 0.4);
  background: rgba(96, 205, 255, 0.04);
}

.add-icon {
  font-size: 28px;
  color: rgba(255, 255, 255, 0.2);
  transition: color 0.15s;
}

.add-card:hover .add-icon {
  color: #60cdff;
}

.add-text {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.25);
  transition: color 0.15s;
}

.add-card:hover .add-text {
  color: rgba(255, 255, 255, 0.6);
}
</style>
