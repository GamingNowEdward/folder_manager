import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from '@/App.vue'
import { useProjectStore } from '@/features/projects'
import { subscribeWorkspaceChanges } from './composables/useWorkspaceSync'

export function bootstrap(): void {
  const app = createApp(App)
  app.use(createPinia())

  const projectStore = useProjectStore()
  app.mount('#app')
  void projectStore.loadFromDisk()
  void subscribeWorkspaceChanges()
}
