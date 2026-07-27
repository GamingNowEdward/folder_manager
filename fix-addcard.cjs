const fs = require('fs');
let c = fs.readFileSync('c:/opencode/folder_manager_fluent/src/App.vue', 'utf8');
c = c.replace(
  '<AddFolderCard @click="onAddFolder" />',
  '<AddFolderCard v-if="store.currentFolders.length === 0" @click="onAddFolder" />'
);
fs.writeFileSync('c:/opencode/folder_manager_fluent/src/App.vue', c, 'utf8');
console.log('Done');