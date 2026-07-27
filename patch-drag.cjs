const fs = require('fs');
let c = fs.readFileSync('c:/opencode/folder_manager_fluent/src/App.vue', 'utf8');

// 1. Add nextTick import
c = c.replace(
  "import { onMounted, onUnmounted, ref, computed } from 'vue'",
  "import { onMounted, onUnmounted, ref, computed, nextTick } from 'vue'"
);

// 2. Add drag state vars
c = c.replace(
  'let dragStartPos = { x: 0, y: 0 }',
  [
    'let dragStartPos = { x: 0, y: 0 }',
    'const dragFolderName = ref(\'\')',
    'const dragGhostX = ref(0)',
    'const dragGhostY = ref(0)',
    'const dragGhostW = ref(0)',
    'const dragGhostH = ref(0)',
    'const dragOffsetX = ref(0)',
    'const dragOffsetY = ref(0)',
    'let flipPending = false',
    '',
    'const dragFolder = computed(() =>',
    '  store.currentFolders.find((f) => f.name === dragFolderName.value) ?? null',
    ')'
  ].join('\n')
);

// 3. Replace getDropTargetIndex
const gStart = c.indexOf('function getDropTargetIndex');
const gEnd = c.indexOf('\n}', gStart) + 2;
c = c.substring(0, gStart) + [
  'function getDropTargetIndex(e: { clientX: number; clientY: number }): number {',
  '  const allCards = document.querySelectorAll(\'[data-folder-index]\')',
  '  const cards: Element[] = []',
  '  allCards.forEach((card) => {',
  '    const n = card.getAttribute(\'data-folder-name\')',
  '    if (n !== dragFolderName.value) cards.push(card)',
  '  })',
  '  if (cards.length === 0) return 0',
  '',
  '  const mx = e.clientX',
  '  const my = e.clientY',
  '  const threshold = 5',
  '',
  '  const rows: number[][] = []',
  '  const rowTops: number[] = []',
  '  const rowBottoms: number[] = []',
  '',
  '  for (let i = 0; i < cards.length; i++) {',
  '    const r = cards[i].getBoundingClientRect()',
  '    const top = Math.round(r.top)',
  '    let rowIdx = rowTops.findIndex((t) => Math.abs(t - top) < threshold)',
  '    if (rowIdx < 0) {',
  '      rowIdx = rows.length',
  '      rows.push([])',
  '      rowTops.push(r.top)',
  '      rowBottoms.push(r.bottom)',
  '    }',
  '    rows[rowIdx].push(i)',
  '    rowBottoms[rowIdx] = Math.max(rowBottoms[rowIdx], r.bottom)',
  '  }',
  '',
  '  for (const row of rows) {',
  '    row.sort((a, b) => cards[a].getBoundingClientRect().left - cards[b].getBoundingClientRect().left)',
  '  }',
  '',
  '  const visualOrder = rows.flat()',
  '',
  '  for (let vi = 0; vi < visualOrder.length; vi++) {',
  '    const idx = visualOrder[vi]',
  '    const rect = cards[idx].getBoundingClientRect()',
  '    if (my >= rect.top && my <= rect.bottom) {',
  '      if (mx < rect.left + rect.width / 2) {',
  '        return parseInt(cards[idx].getAttribute(\'data-folder-index\') ?? \'0\')',
  '      }',
  '      continue',
  '    }',
  '    if (my < rect.top) {',
  '      return parseInt(cards[idx].getAttribute(\'data-folder-index\') ?? \'0\')',
  '    }',
  '  }',
  '',
  '  return store.currentFolders.length',
  '}',
].join('\n') + c.substring(gEnd);

// 4. Add liveReorder + flipAnimate before onConfirmOk
c = c.replace(
  'function onConfirmOk(): void {',
  [
    'function liveReorder(fromIndex: number, toIndex: number): boolean {',
    '  const proj = store.currentProject',
    '  if (!proj || !dragFolderName.value) return false',
    '  const folders = proj.folders',
    '  const curIdx = folders.findIndex((f) => f.name === dragFolderName.value)',
    '  if (curIdx < 0) return false',
    '  const adjTo = toIndex > curIdx ? toIndex - 1 : toIndex',
    '  if (adjTo === curIdx) return false',
    '  const [moved] = folders.splice(curIdx, 1)',
    '  folders.splice(adjTo, 0, moved)',
    '  return true',
    '}',
    '',
    'function flipAnimate(excludeName: string): void {',
    '  const grid = cardsAreaRef.value?.querySelector(\'.cards-grid\')',
    '  if (!grid) return',
    '  const els = grid.querySelectorAll(\'[data-folder-index]\')',
    '  const firstRects = new Map<string, DOMRect>()',
    '  els.forEach((el) => {',
    '    const name = el.getAttribute(\'data-folder-name\')',
    '    if (name && name !== excludeName) {',
    '      firstRects.set(name, el.getBoundingClientRect())',
    '    }',
    '  })',
    '  nextTick(() => {',
    '    const newEls = grid.querySelectorAll(\'[data-folder-index]\')',
    '    newEls.forEach((el) => {',
    '      const name = el.getAttribute(\'data-folder-name\')',
    '      if (!name || name === excludeName) return',
    '      const first = firstRects.get(name)',
    '      if (!first) return',
    '      const last = el.getBoundingClientRect()',
    '      const dx = first.left - last.left',
    '      const dy = first.top - last.top',
    '      if (Math.abs(dx) > 1 || Math.abs(dy) > 1) {',
    '        ;(el as HTMLElement).animate(',
    '          [',
    '            { transform: `translate(${dx}px, ${dy}px)` },',
    '            { transform: \'translate(0, 0)\' }',
    '          ],',
    '          { duration: 200, easing: \'cubic-bezier(0.2, 0, 0, 1)\' }',
    '        )',
    '      }',
    '    })',
    '  })',
    '}',
    '',
    'function onConfirmOk(): void {',
  ].join('\n')
);

// 5. Replace onWindowMouseMove drag section
c = c.replace(
  [
    '  if (dragFromIndex.value >= 0) {',
    '    const dx = Math.abs(e.clientX - dragStartPos.x)',
    '    const dy = Math.abs(e.clientY - dragStartPos.y)',
    '    if (dx > 5 || dy > 5) {',
    '      if (!isDraggingCard.value) {',
    '        isDraggingCard.value = true',
    "        document.body.style.cursor = 'grabbing'",
    '      }',
    '      dragToIndex.value = getDropTargetIndex(e)',
    '      dropIndicatorIndex.value = dragToIndex.value',
    '    }',
    '  }',
  ].join('\n'),
  [
    '  if (dragFromIndex.value >= 0) {',
    '    const dx = Math.abs(e.clientX - dragStartPos.x)',
    '    const dy = Math.abs(e.clientY - dragStartPos.y)',
    '    if (dx > 5 || dy > 5) {',
    '      if (!isDraggingCard.value) {',
    '        isDraggingCard.value = true',
    "        document.body.style.cursor = 'grabbing'",
    '        const el = document.querySelector(',
    '          `[data-folder-name="${dragFolderName.value}"]`',
    '        )',
    '        if (el) {',
    '          const rect = el.getBoundingClientRect()',
    '          dragGhostW.value = rect.width',
    '          dragGhostH.value = rect.height',
    '          dragOffsetX.value = e.clientX - rect.left',
    '          dragOffsetY.value = e.clientY - rect.top',
    '        }',
    '      }',
    '      dragGhostX.value = e.clientX - dragOffsetX.value',
    '      dragGhostY.value = e.clientY - dragOffsetY.value',
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
    '    }',
    '  }',
  ].join('\n')
);

// 6. Replace onWindowMouseUp
c = c.replace(
  [
    'function onWindowMouseUp(e: MouseEvent): void {',
    '  if (isBoxSelecting.value) {',
    '    isBoxSelecting.value = false',
    '    boxRect.value = null',
    '    return',
    '  }',
    '',
    '  if (dragFromIndex.value >= 0) {',
    '    if (isDraggingCard.value) {',
    '      if (dragToIndex.value >= 0 && dragToIndex.value !== dragFromIndex.value) {',
    '        store.reorderFolders(dragFromIndex.value, dragToIndex.value)',
    "        store.setStatus('已重新排序')",
    '      }',
    '    } else {',
    '      const folder = store.currentFolders[dragFromIndex.value]',
    '      if (folder) onCardSelect(folder.name, e.ctrlKey || e.metaKey)',
    '    }',
    '    dragFromIndex.value = -1',
    '    dragToIndex.value = -1',
    '    isDraggingCard.value = false',
    '    dropIndicatorIndex.value = -1',
    "    document.body.style.cursor = ''",
    '  }',
    '}',
  ].join('\n'),
  [
    'function onWindowMouseUp(e: MouseEvent): void {',
    '  if (isBoxSelecting.value) {',
    '    isBoxSelecting.value = false',
    '    boxRect.value = null',
    '    return',
    '  }',
    '',
    '  if (dragFromIndex.value >= 0) {',
    '    if (isDraggingCard.value) {',
    '      store._save()',
    "      store.setStatus('已重新排序')",
    '    } else {',
    '      const folder = store.currentFolders[dragFromIndex.value]',
    '      if (folder) onCardSelect(folder.name, e.ctrlKey || e.metaKey)',
    '    }',
    '    dragFromIndex.value = -1',
    '    dragToIndex.value = -1',
    '    isDraggingCard.value = false',
    '    dropIndicatorIndex.value = -1',
    '    dragFolderName.value = \'\'',
    "    document.body.style.cursor = ''",
    '  }',
    '}',
  ].join('\n')
);

// 7. Replace onCardMouseDown
c = c.replace(
  [
    'function onCardMouseDown(index: number, _e: MouseEvent): void {',
    '  dragFromIndex.value = index',
    '  dragStartPos = { x: _e.clientX, y: _e.clientY }',
    '  isDraggingCard.value = false',
    '}',
  ].join('\n'),
  [
    'function onCardMouseDown(index: number, _e: MouseEvent): void {',
    '  dragFromIndex.value = index',
    '  dragStartPos = { x: _e.clientX, y: _e.clientY }',
    '  isDraggingCard.value = false',
    '  dragFolderName.value = store.currentFolders[index]?.name ?? \'\'',
    '}',
  ].join('\n')
);

// 8. Update FolderCard template
c = c.replace(
  '              <FolderCard\n                :class="{ \'is-drop-target\': dropIndicatorIndex === index }"',
  '              <FolderCard\n                :class="{ \'is-drag-source\': isDraggingCard && folder.name === dragFolderName }"'
);

// 9. Add data-folder-name attr
c = c.replace(
  '                :folder="folder"',
  '                :data-folder-name="folder.name"\n                :folder="folder"'
);

// 10. Add ghost element before closing cards-area div
c = c.replace(
  '          </div>\n          <div v-else class="empty-hint">',
  [
    '          </div>',
    '          <div',
    '            v-if="isDraggingCard && dragFolder"',
    '            class="drag-ghost"',
    '            :style="{',
    '              left: dragGhostX + \'px\',',
    '              top: dragGhostY + \'px\',',
    '              width: dragGhostW + \'px\',',
    '              height: dragGhostH + \'px\'',
    '            }"',
    '          >',
    '            <div class="ghost-icon">📂</div>',
    '            <div class="ghost-name">{{ dragFolder.name }}</div>',
    '            <div class="ghost-path">{{ dragFolder.path }}</div>',
    '          </div>',
    '          <div v-else class="empty-hint">',
  ].join('\n')
);

// 11. Add styles at end
c += [
  '',
  '<style scoped>',
  '.drag-ghost {',
  '  position: fixed;',
  '  z-index: 9999;',
  '  pointer-events: none;',
  '  background: rgba(44, 44, 48, 0.95);',
  '  border: 1px solid rgba(96, 205, 255, 0.35);',
  '  border-radius: 14px;',
  '  padding: 16px;',
  '  display: flex;',
  '  flex-direction: column;',
  '  gap: 4px;',
  '  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.45);',
  '  transform: rotate(2deg) scale(1.04);',
  '}',
  '',
  '.ghost-icon {',
  '  font-size: 17px;',
  '  margin-bottom: 2px;',
  '}',
  '',
  '.ghost-name {',
  '  font-size: 14px;',
  '  font-weight: 600;',
  '  color: rgba(255, 255, 255, 0.9);',
  '}',
  '',
  '.ghost-path {',
  '  font-size: 11px;',
  '  color: rgba(255, 255, 255, 0.35);',
  '  overflow: hidden;',
  '  text-overflow: ellipsis;',
  '  white-space: nowrap;',
  '}',
  '</style>',
  '',
].join('\n');

fs.writeFileSync('c:/opencode/folder_manager_fluent/src/App.vue', c);
console.log('Patched OK');