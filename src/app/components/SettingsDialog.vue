<script setup lang="ts">
import { ref, watch } from 'vue'
import type { ApiSettings } from '@/infrastructure/tauri/settings'

const props = defineProps<{
  settings: ApiSettings | null
  errorMessage: string
  loading: boolean
  saving: boolean
}>()

const emit = defineEmits<{
  // 注意：`<input type="number">` 的 v-model 在编辑后给出 number，未编辑时是 string
  save: [port: string | number]
  copy: []
  cancel: []
}>()

const portInput = ref('')

watch(
  () => props.settings,
  (current) => {
    // 预填「期望端口」：保存过的用它，否则用默认端口（当前实际监听单独显示）
    if (current) portInput.value = String(current.savedPort ?? current.defaultPort)
  },
  { immediate: true }
)
</script>

<template>
  <div class="dialog-overlay" @click.self="emit('cancel')">
    <div class="dialog-panel">
      <h3 class="dialog-title">设置</h3>

      <div class="dialog-field">
        <label class="dialog-label">本地 HTTP API</label>
        <div class="settings-current">
          当前监听：<code>{{ settings?.url ?? '未启动' }}</code>
        </div>
        <div v-if="settings?.envOverride" class="settings-hint">
          端口由环境变量 FOLDER_MANAGER_API_PORT 覆盖，移除该变量后界面设置才会生效。
        </div>
        <div v-else-if="settings?.restartRequired" class="settings-hint warn">
          已保存端口 {{ settings.savedPort }}，重启应用后生效。
        </div>
        <div
          v-else-if="
            settings &&
            settings.startupPort !== null &&
            settings.actualPort !== null &&
            settings.startupPort !== settings.actualPort
          "
          class="settings-hint warn"
        >
          端口 {{ settings.startupPort }} 被占用，已自动顺延到 {{ settings.actualPort }}。
        </div>
      </div>

      <div class="dialog-field">
        <label class="dialog-label">API 端口（1–65535）</label>
        <div class="port-row">
          <input
            v-model="portInput"
            class="dialog-input"
            type="number"
            min="1"
            max="65535"
            :disabled="loading || saving"
            @keydown.enter="emit('save', portInput)"
          />
          <button class="btn-ok" :disabled="loading || saving" @click="emit('save', portInput)">
            保存
          </button>
        </div>
        <div v-if="errorMessage" class="settings-error">{{ errorMessage }}</div>
      </div>

      <div class="dialog-field">
        <label class="dialog-label">给 AI Agent 使用</label>
        <button class="btn" @click="emit('copy')">📋 复制 Agent 接入文档</button>
        <div class="settings-hint">
          复制 docs/API.md 全文（附当前监听地址），直接粘贴给 Agent 即可。
        </div>
      </div>

      <div class="dialog-buttons">
        <button class="btn-cancel" @click="emit('cancel')">关闭</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-current {
  font-size: 13px;
  color: rgba(255, 255, 255, 0.7);
}

.settings-current code {
  color: #60cdff;
  font-family: 'Cascadia Code', 'Consolas', monospace;
  font-size: 12px;
  user-select: text;
}

.settings-hint {
  margin-top: 8px;
  font-size: 11px;
  line-height: 1.6;
  color: rgba(255, 255, 255, 0.36);
}

.settings-hint.warn {
  color: rgba(255, 200, 120, 0.75);
}

.settings-error {
  margin-top: 8px;
  font-size: 12px;
  color: #ff99a4;
}

.port-row {
  display: flex;
  gap: 8px;
}

.port-row .dialog-input {
  flex: 1;
}

/* 端口输入：隐藏数字原生上下微调控件，保持纯手动输入 */
.port-row input[type='number']::-webkit-outer-spin-button,
.port-row input[type='number']::-webkit-inner-spin-button {
  -webkit-appearance: none;
  appearance: none;
  margin: 0;
}
</style>
