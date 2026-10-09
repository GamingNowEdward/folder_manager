<script setup lang="ts">
import { useDialogs } from '@/features/dialogs'
import { useProjectActions, useProjectStore } from '@/features/projects'
import { useFolderActions } from '@/features/folders'
import ProjectDialog from '@/features/projects/components/ProjectDialog.vue'
import FolderDialog from '@/features/folders/components/FolderDialog.vue'
import ConfirmDialog from '@/features/dialogs/components/ConfirmDialog.vue'
import SettingsDialog from '@/app/components/SettingsDialog.vue'
import { useSettings } from '@/app/composables/useSettings'

const dialogs = useDialogs()
const projectStore = useProjectStore()
const projectActions = useProjectActions()
const folderActions = useFolderActions()
const settingsActions = useSettings()
const {
  settings: apiSettings,
  errorMessage: settingsError,
  loading: settingsLoading,
  saving: settingsSaving
} = settingsActions
const { projectDialog, folderDialog, confirmDialog, settingsDialog } = dialogs
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
  <SettingsDialog
    v-if="settingsDialog.visible"
    :settings="apiSettings"
    :error-message="settingsError"
    :loading="settingsLoading"
    :saving="settingsSaving"
    @save="settingsActions.save"
    @copy="settingsActions.copyDocumentation"
    @cancel="settingsActions.close"
  />
</template>
