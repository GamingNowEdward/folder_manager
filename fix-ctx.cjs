const fs = require('fs');
let c = fs.readFileSync('c:/opencode/folder_manager_fluent/src/App.vue', 'utf8');

c = c.replace(
  "onMounted(() => {\n  store.loadFromDisk()",
  "onMounted(() => {\n  document.addEventListener('contextmenu', (e) => {\n    if (!(e.target as HTMLElement).closest('input, textarea')) {\n      e.preventDefault()\n    }\n  })\n  store.loadFromDisk()"
);

fs.writeFileSync('c:/opencode/folder_manager_fluent/src/App.vue', c, 'utf8');
console.log('Done');