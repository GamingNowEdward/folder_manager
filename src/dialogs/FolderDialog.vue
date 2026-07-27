<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'

const props = defineProps<{
  mode: 'add' | 'edit'
  defaultName?: string
  defaultPath?: string
}>()

const emit = defineEmits<{
  confirm: [result: { name: string; path: string }]
  cancel: []
}>()

const name = ref(props.defaultName ?? '')
const path = ref(props.defaultPath ?? '')
const nameInputRef = ref<HTMLInputElement | null>(null)

onMounted(() => {
  setTimeout(() => {
    nameInputRef.value?.select()
  }, 50)
})

async function onBrowse(): Promise<void> {
  const selected = await open({ directory: true, multiple: false })
  if (selected) {
    path.value = selected
  }
}

function onSubmit(): void {
  const trimmedPath = path.value.trim()
  if (!trimmedPath) return
  let trimmedName = name.value.trim()
  if (!trimmedName) {
    trimmedName = trimmedPath.replace(/[/\\]+$/, '').split(/[/\\]/).pop() ?? ''
  }
  emit('confirm', { name: trimmedName, path: trimmedPath })
}
</script>

<template>
  <div class="dialog-overlay" @click.self="emit('cancel')">
    <div class="dialog-panel">
      <h3 class="dialog-title">{{ mode === 'add' ? '添加文件夹' : '编辑文件夹' }}</h3>
      <div class="dialog-field">
        <label class="dialog-label">名称</label>
        <input ref="nameInputRef" v-model="name" class="dialog-input" placeholder="留空则自动取路径末段" @keydown.enter="onSubmit" />
      </div>
      <div class="dialog-field">
        <label class="dialog-label">路径</label>
        <div class="path-row">
          <input v-model="path" class="dialog-input" placeholder="选择或输入文件夹路径" @keydown.enter="onSubmit" />
          <button class="btn-browse" @click="onBrowse">浏览...</button>
        </div>
      </div>
      <div class="dialog-buttons">
        <button class="btn-ok" @click="onSubmit">确定</button>
        <button class="btn-cancel" @click="emit('cancel')">取消</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.dialog-field {
  margin-bottom: 14px;
}

.path-row {
  display: flex;
  gap: 8px;
}

.path-row .dialog-input {
  flex: 1;
}
</style>
