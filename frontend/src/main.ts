// ponytail: worker 副作用必须在 monaco 任何使用之前；ConfigEditor.vue 走 '@/utils/monaco-env' 复用同一份 monaco 实例
import './utils/monaco-env'
import { createApp } from 'vue'
import { createPinia } from 'pinia'
import piniaPluginPersistedstate from 'pinia-plugin-persistedstate'
import ElementPlus from 'element-plus'
import 'element-plus/dist/index.css'
import 'element-plus/theme-chalk/dark/css-vars.css'
import * as ElementPlusIconsVue from '@element-plus/icons-vue'

import App from './App.vue'
import router from './router'
import i18n from './i18n'
import { useSettingsStore } from './stores/settings'
import OnIconPlugin from './components/OnIcon'

const app = createApp(App)

// 注册Element Plus图标
for (const [key, component] of Object.entries(ElementPlusIconsVue)) {
  app.component(key, component)
}

const pinia = createPinia()
pinia.use(piniaPluginPersistedstate)

app.use(pinia)
app.use(router)
app.use(ElementPlus)
app.use(OnIconPlugin)
app.use(i18n)

// 从持久化 store 恢复主题设置到 DOM
const settingsStore = useSettingsStore()
settingsStore.initTheme()

app.mount('#app')
