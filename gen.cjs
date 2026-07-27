const fs = require('fs');
const original = fs.readFileSync('c:/opencode/folder_manager_tauri/src/App.vue', 'utf8');
let content = original
  .replace("import ProjectSelect from './components/ProjectSelect.vue'", "import Sidebar from './components/Sidebar.vue'")
  .replace("import FlowLayout from './components/FlowLayout.vue'\n", '');
const templateStart = content.indexOf('<template>');
const scriptSection = content.substring(0, templateStart);
const newTemplate = `<template>
  <div class="app-root">
    <TitleBar />
    <div class="app-body">
      <Sidebar
        :projects="store.projects"
        :current-name="store.currentProjectName"
        @select="store.selectProject"
        @add="onAddProject"
        @edit="onEditProject"
        @delete="onDeleteProject"
      />
      <div class="main-area">
        <div class="main-header" v-if="store.currentProject">
          <span class="main-title">{{ store.currentProjectName }}</span>
          <div class="header-actions">
            <button class="btn btn-accent" @click="onAddFolder">\uFF0B \u6DFB\u52A0\u6587\u4EF6\u5939</button>
          </div>
        </div>
        <div ref="cardsAreaRef" class="cards-area" @mousedown="onCardsAreaMouseDown">
          <div v-if="isBoxSelecting && boxRect" class="box-select-overlay" :style="boxOverlayStyle" />
          <div v-if="store.currentProject" class="cards-grid">
            <template v-for="(folder, index) in store.currentFolders" :key="folder.name">
              <FolderCard
                :class="{ 'is-drop-target': dropIndicatorIndex === index }"
                :folder="folder"
                :index="index"
                :total="store.currentFolders.length"
                :selected="selectedFolderNames.includes(folder.name)"
                @mousedown="onCardMouseDown"
                @open="onOpenFolder"
                @copy="onCopyPath"
                @edit="onEditFolder(folder.name, folder.path)"
                @move-up="store.moveFolder(folder.name, -1)"
                @move-down="store.moveFolder(folder.name, 1)"
                @delete="onDeleteFolder"
              />
            </template>
            <div v-if="dropIndicatorIndex === store.currentFolders.length" class="drop-indicator-end" />
            <AddFolderCard @click="onAddFolder" />
          </div>
          <div v-else class="empty-hint">
            <div class="empty-icon">\uD83D\uDCC1</div>
            <div class="empty-text">\u9009\u62E9\u6216\u65B0\u5EFA\u4E00\u4E2A\u9879\u76EE\u5F00\u59CB</div>
          </div>
        </div>
      </div>
    </div>
    <StatusBar />
    <ProjectDialog
      v-if="showProjectDialog"
      :mode="projectDialogMode"
      :default-name="projectDialogMode === 'edit' ? store.currentProjectName : ''"
      @confirm="onProjectDialogConfirm"
      @cancel="showProjectDialog = false"
    />
    <FolderDialog
      v-if="showFolderDialog"
      :mode="folderDialogMode"
      :default-name="editFolderName"
      :default-path="editFolderPath"
      @confirm="onFolderDialogConfirm"
      @cancel="showFolderDialog = false"
    />
    <ConfirmDialog
      v-if="showConfirmDialog"
      :title="confirmDialogTitle"
      :message="confirmDialogMessage"
      @confirm="onConfirmOk"
      @cancel="onConfirmCancel"
    />
  </div>
</template>
`;
fs.writeFileSync('c:/opencode/folder_manager_fluent/src/App.vue', scriptSection + newTemplate);
console.log('Done');