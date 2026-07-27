const fs = require('fs');
let c = fs.readFileSync('c:/opencode/folder_manager_fluent/src/App.vue', 'utf8');

c = c.replace(
  '  background: rgba(44, 44, 48, 0.95);',
  '  background: rgba(40, 40, 46, 0.72);\n  backdrop-filter: blur(12px);'
);

fs.writeFileSync('c:/opencode/folder_manager_fluent/src/App.vue', c, 'utf8');
console.log('Done');