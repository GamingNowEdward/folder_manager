<script setup lang="ts">
import { useDialogs } from '@/features/dialogs'
import { useProjectActions, useProjectStore } from '@/features/projects'
import { useFolderActions } from '@/features/folders'
import ProjectDialog from '@/features/projects/components/ProjectDialog.vue'
import FolderDialog from '@/features/folders/components/FolderDialog.vue'
import ConfirmDialog from '@/features/dialogs/components/ConfirmDialog.vue'

const dialogs = useDialogs()
const projectStore = useProjectStore()
const projectActions = useProjectActions()
const folderActions = useFolderActions()
const { projectDialog, folderDialog, confirmDialog } = dialogs
</script>

<template>
  <ProjectDialog
    v-if="projectDialog.visible"
    :mode="projectDialog.mode"
    :default-name="projectDialog.mode === 'edit' ? (projectStore.currentProject?.name ?? '') : ''"
    @confirm="projectActions.submitName"
    @cancel="dialogs.closeProjectDialog"
  />
  <FolderDialog
    v-if="folderDialog.visible"
    :mode="folderDialog.mode"
    :default-name="folderDialog.name"
    :default-path="folderDialog.path"
    @confirm="folderActions.submitForm"
    @cancel="dialogs.closeFolderDialog"
  />
  <ConfirmDialog
    v-if="confirmDialog.visible"
    :title="confirmDialog.title"
    :message="confirmDialog.message"
    @confirm="dialogs.confirm"
    @cancel="dialogs.cancelConfirm"
  />
</template>
