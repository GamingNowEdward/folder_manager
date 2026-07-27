<script setup lang="ts">
import { ref, onMounted } from 'vue'

const props = defineProps<{
  mode: 'add' | 'edit'
  defaultName?: string
}>()

const emit = defineEmits<{
  confirm: [name: string]
  cancel: []
}>()

const name = ref(props.defaultName ?? '')
const inputRef = ref<HTMLInputElement | null>(null)

onMounted(() => {
  setTimeout(() => {
    inputRef.value?.select()
  }, 50)
})

function onSubmit(): void {
  const trimmed = name.value.trim()
  if (!trimmed) return
  emit('confirm', trimmed)
}
</script>

<template>
  <div class="dialog-overlay" @click.self="emit('cancel')">
    <div class="dialog-panel" style="width: 360px;">
      <h3 class="dialog-title">{{ mode === 'add' ? '新建项目' : '编辑项目' }}</h3>
      <label class="dialog-label">项目名称</label>
      <input
        ref="inputRef"
        v-model="name"
        class="dialog-input"
        placeholder="请输入项目名称"
        @keydown.enter="onSubmit"
      />
      <div class="dialog-buttons">
        <button class="btn-ok" @click="onSubmit">确定</button>
        <button class="btn-cancel" @click="emit('cancel')">取消</button>
      </div>
    </div>
  </div>
</template>
