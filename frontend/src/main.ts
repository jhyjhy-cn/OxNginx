// ponytail: worker 副作用必须在 monaco 任何使用之前；ConfigEditor.vue 走 '@/utils/monaco-env' 复用同一份 monaco 实例
import './utils/monaco-env'
import { createApp } from 'vue'
import { createPinia } from 'pinia'
import piniaPluginPersistedstate from 'pinia-plugin-persistedstate'
// ponytail: element-plus 由 unplugin-vue-components 的 ElementPlusResolver 自动按需注册，不在此处 use
// dark/css-vars.css 提供暗色变量基础（settings.darkMode 切 html.dark 触发）
import 'element-plus/theme-chalk/dark/css-vars.css'
import './styles/main.css'
import * as ElementPlusIconsVue from '@element-plus/icons-vue'

import App from './App.vue'
import router, { restoreDynamicRoutes } from './router'
import i18n, { restoreI18n, restoreLocale } from './i18n'
import { useSettingsStore } from './stores/settings'
import OnIconPlugin from './components/OnIcon'
import { vAuth } from './directives/auth'

const app = createApp(App)

// 注册Element Plus图标
for (const [key, component] of Object.entries(ElementPlusIconsVue)) {
  app.component(key, component)
}

const pinia = createPinia()
pinia.use(piniaPluginPersistedstate)

app.use(pinia)
// 必须在 app.use(router) 之前恢复动态路由——router.install() 内部会立即解析初始路由
restoreDynamicRoutes()
app.use(router)
app.use(OnIconPlugin)
app.use(i18n)

// 从持久化 store 恢复主题设置到 DOM
const settingsStore = useSettingsStore()
settingsStore.initTheme()
restoreLocale()
restoreI18n()

app.directive('auth', vAuth)
app.mount('#app')