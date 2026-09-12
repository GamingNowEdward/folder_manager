<script setup lang="ts">
import { storeToRefs } from 'pinia'
import { useProjectActions, useProjectStore } from '@/features/projects'
import Sidebar from '@/features/projects/components/Sidebar.vue'
import FolderWorkspace from '@/features/folders/components/FolderWorkspace.vue'
import TitleBar from './components/TitleBar.vue'
import StatusBar from './components/StatusBar.vue'
import DialogHost from './components/DialogHost.vue'

const projectStore = useProjectStore()
const projectActions = useProjectActions()
const { projects, currentProjectId } = storeToRefs(projectStore)
</script>

<template>
  <div class="app-root">
    <TitleBar />
    <div class="app-body">
      <Sidebar
        :projects="projects"
        :current-id="currentProjectId"
        @select="projectStore.selectProject"
        @add="projectActions.openAddDialog"
        @edit="projectActions.openEditDialog"
        @delete="projectActions.requestDelete"
      />
      <FolderWorkspace />
    </div>
    <StatusBar />
    <DialogHost />
  </div>
</template>
