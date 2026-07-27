const fs = require('fs');
let c = fs.readFileSync('c:/opencode/folder_manager_fluent/src/App.vue', 'utf8');

// Add blur handler registration in onMounted
c = c.replace(
  "  document.addEventListener('keydown', onWindowKeyDown)",
  "  document.addEventListener('keydown', onWindowKeyDown)\n  window.addEventListener('blur', onWindowBlur)"
);

// Add blur handler cleanup in onUnmounted
c = c.replace(
  "  document.removeEventListener('keydown', onWindowKeyDown)",
  "  document.removeEventListener('keydown', onWindowKeyDown)\n  window.removeEventListener('blur', onWindowBlur)"
);

// Add the onWindowBlur function before onWindowKeyDown
c = c.replace(
  'function onWindowKeyDown(e: KeyboardEvent): void {',
  `function onWindowBlur(): void {
  if (isDraggingCard.value || dragFromIndex.value >= 0) {
    dragFromIndex.value = -1
    dragToIndex.value = -1
    isDraggingCard.value = false
    dropIndicatorIndex.value = -1
    dragFolderName.value = ''
    document.body.style.cursor = ''
  }
  if (isBoxSelecting.value) {
    isBoxSelecting.value = false
    boxRect.value = null
  }
}

function onWindowKeyDown(e: KeyboardEvent): void {`
);

fs.writeFileSync('c:/opencode/folder_manager_fluent/src/App.vue', c, 'utf8');
console.log('Done');