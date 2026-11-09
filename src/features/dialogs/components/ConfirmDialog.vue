<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useDialogKeys } from '../composables/useDialogKeys'

defineProps<{
  title: string
  message: string
}>()

const emit = defineEmits<{
  confirm: []
  cancel: []
}>()

const confirmButton = ref<HTMLButtonElement | null>(null)

useDialogKeys({
  onConfirm: () => emit('confirm'),
  onCancel: () => emit('cancel'),
  allowQuickKeys: true
})

// 默认聚焦「确定」：既是视觉上的默认按钮，也让鼠标用户直接回车确认。
onMounted(() => confirmButton.value?.focus())
</script>

<template>
  <div class="dialog-overlay" @click.self="emit('cancel')">
    <div class="dialog-panel">
      <h3 class="dialog-title">{{ title }}</h3>
      <p class="dialog-message">{{ message }}</p>
      <div class="dialog-buttons">
        <button ref="confirmButton" class="btn-ok btn-confirm-danger" @click="emit('confirm')">
          确定<span class="dialog-key-hint">Y</span>
        </button>
        <button class="btn-cancel" @click="emit('cancel')">
          取消<span class="dialog-key-hint">N</span>
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.btn-confirm-danger {
  background: rgba(255, 153, 164, 0.15);
  border-color: rgba(255, 153, 164, 0.4);
  color: #ff99a4;
}

.btn-confirm-danger:hover {
  background: rgba(255, 153, 164, 0.25);
}

/* 快捷键提示与按钮文字用 flex 精确垂直居中：纯 baseline 对齐时小字号会明显偏下 */
.btn-confirm-danger,
.btn-cancel {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 5px;
}

.dialog-key-hint {
  font-size: 10px;
  font-weight: 400;
  line-height: 1;
  opacity: 0.55;
  /* 光学校准：flex 几何居中后，小字号拉丁字母的视觉重心比汉字略高，下移 0.5px 抵消 */
  position: relative;
  top: 0.5px;
}
</style>
