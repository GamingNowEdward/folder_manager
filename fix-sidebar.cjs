const fs = require('fs');
let c = fs.readFileSync('c:/opencode/folder_manager_fluent/src/components/Sidebar.vue', 'utf8');

c = c.replace(
  '      <div class="sidebar-item" @click="emit(\'add\')">＋ 新建项目</div>',
  '      <div class="sidebar-item" @click="emit(\'add\')"><span class="action-icon">＋</span>新建项目</div>'
);
c = c.replace(
  '✎ 编辑</div>',
  '<span class="action-icon">✎</span>编辑</div>'
);
c = c.replace(
  '✕ 删除</div>',
  '<span class="action-icon">✕</span>删除</div>'
);

c = c.replace(
  '.sidebar-item.danger:hover {',
  '.action-icon {\n  width: 18px;\n  text-align: center;\n  flex-shrink: 0;\n}\n\n.sidebar-item.danger:hover {'
);

fs.writeFileSync('c:/opencode/folder_manager_fluent/src/components/Sidebar.vue', c, 'utf8');
console.log('Fixed');