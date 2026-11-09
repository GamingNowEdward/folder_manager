<script setup lang="ts">
import { usePreferencesStore } from '@/shared/stores/preferences.store'
import { useStatusStore } from '@/shared/stores/status.store'

const statusStore = useStatusStore()
const preferences = usePreferencesStore()
</script>

<template>
  <div class="status-bar">
    <span class="status-text">{{ statusStore.message }}</span>
    <div class="view-toggle" role="group" aria-label="文件夹视图">
      <button
        class="view-btn"
        :class="{ active: preferences.viewMode === 'grid' }"
        :aria-pressed="preferences.viewMode === 'grid'"
        title="卡片视图"
        @click="preferences.setViewMode('grid')"
      >
        <svg
          width="14"
          height="14"
          viewBox="0 0 16 16"
          fill="none"
          stroke="currentColor"
          stroke-width="1.3"
        >
          <rect x="2.2" y="2.2" width="4.6" height="4.6" rx="1.2" />
          <rect x="9.2" y="2.2" width="4.6" height="4.6" rx="1.2" />
          <rect x="2.2" y="9.2" width="4.6" height="4.6" rx="1.2" />
          <rect x="9.2" y="9.2" width="4.6" height="4.6" rx="1.2" />
        </svg>
      </button>
      <button
        class="view-btn"
        :class="{ active: preferences.viewMode === 'list' }"
        :aria-pressed="preferences.viewMode === 'list'"
        title="列表视图"
        @click="preferences.setViewMode('list')"
      >
        <svg
          width="14"
          height="14"
          viewBox="0 0 16 16"
          fill="none"
          stroke="currentColor"
          stroke-width="1.3"
        >
          <rect x="2.2" y="2.6" width="3" height="3" rx="0.8" />
          <line x1="7.6" y1="4.1" x2="13.8" y2="4.1" stroke-linecap="round" />
          <rect x="2.2" y="6.6" width="3" height="3" rx="0.8" />
          <line x1="7.6" y1="8.1" x2="13.8" y2="8.1" stroke-linecap="round" />
          <rect x="2.2" y="10.6" width="3" height="3" rx="0.8" />
          <line x1="7.6" y1="12.1" x2="13.8" y2="12.1" stroke-linecap="round" />
        </svg>
      </button>
    </div>
  </div>
</template>

<style scoped>
.status-bar {
  border-top: 1px solid rgba(255, 255, 255, 0.06);
  color: rgba(255, 255, 255, 0.3);
  font-size: 11px;
  padding: 2px 10px 2px 16px;
  min-height: 27px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.status-text {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.view-toggle {
  display: flex;
  gap: 2px;
  padding: 2px;
  border-radius: 7px;
  background: rgba(255, 255, 255, 0.04);
  flex-shrink: 0;
}

.view-btn {
  width: 26px;
  height: 20px;
  padding: 0;
  border: none;
  border-radius: 5px;
  background: transparent;
  color: rgba(255, 255, 255, 0.35);
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  transition:
    background 0.12s,
    color 0.12s;
  font-family: inherit;
}

.view-btn:hover {
  background: rgba(255, 255, 255, 0.07);
  color: rgba(255, 255, 255, 0.7);
}

.view-btn.active {
  background: rgba(96, 205, 255, 0.14);
  color: #60cdff;
}

.view-btn:focus-visible {
  outline: 2px solid rgba(96, 205, 255, 0.65);
  outline-offset: 1px;
}
</style>
