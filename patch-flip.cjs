const fs = require('fs');
let c = fs.readFileSync('c:/opencode/folder_manager_fluent/src/App.vue', 'utf8');

// 1. Remove flipAnimate function
const flipStart = c.indexOf('function flipAnimate');
const flipEnd = c.indexOf('\n}', flipStart) + 2;
c = c.substring(0, flipStart) + c.substring(flipEnd);

// 2. Remove flipPending var
c = c.replace('let flipPending = false\n', '');

// 3. Simplify mousemove drag section - remove RAF/flip, just liveReorder
c = c.replace(
  [
    '      const target = getDropTargetIndex(e)',
    '      if (target !== dragToIndex.value) {',
    '        dragToIndex.value = target',
    '        if (!flipPending) {',
    '          flipPending = true',
    '          requestAnimationFrame(() => {',
    '            flipAnimate(dragFolderName.value)',
    '            liveReorder(dragFromIndex.value, dragToIndex.value)',
    '            flipPending = false',
    '          })',
    '        }',
    '      }',
  ].join('\n'),
  [
    '      const target = getDropTargetIndex(e)',
    '      if (target !== dragToIndex.value) {',
    '        dragToIndex.value = target',
    '        liveReorder(dragFromIndex.value, dragToIndex.value)',
    '      }',
  ].join('\n')
);

// 4. Replace template: wrap cards in TransitionGroup
c = c.replace(
  [
    '          <div v-if="store.currentProject" class="cards-grid">',
    '            <template v-for="(folder, index) in store.currentFolders" :key="folder.name">',
    '              <FolderCard',
  ].join('\n'),
  [
    '          <div v-if="store.currentProject" class="cards-grid">',
    '            <TransitionGroup name="flip-list">',
    '              <FolderCard',
    '                v-for="(folder, index) in store.currentFolders"',
    '                :key="folder.name"',
  ].join('\n')
);

c = c.replace(
  [
    '              />',
    '            </template>',
  ].join('\n'),
  [
    '              />',
    '            </TransitionGroup>',
  ].join('\n')
);

// 5. Fix ghost v-if and empty-hint (separate them properly)
c = c.replace(
  '          <div v-else class="empty-hint">',
  '          <div v-if="!store.currentProject" class="empty-hint">'
);

// 6. Add TransitionGroup CSS to the existing <style scoped>
c = c.replace(
  '<style scoped>\n.drag-ghost {',
  [
    '<style scoped>',
    '.flip-list-move {',
    '  transition: transform 0.25s cubic-bezier(0.2, 0, 0, 1);',
    '}',
    '',
    '.drag-ghost {',
  ].join('\n')
);

fs.writeFileSync('c:/opencode/folder_manager_fluent/src/App.vue', c);
console.log('Patched OK');