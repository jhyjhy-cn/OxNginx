// ponytail: worker 副作用必须在 monaco 任何使用之前；ConfigEditor.vue 走 '@/utils/monaco-env' 复用同一份 monaco 实例
import './utils/monaco-env'
import { createApp } from 'vue'
import { createPinia } from 'pinia'
import piniaPluginPersistedstate from 'pinia-plugin-persistedstate'
// ponytail: element-plus 由 unplugin-vue-components 的 ElementPlusResolver 自动按需注册，不在此处 use
// 导入顺序：Tailwind preflight (main.css) → EP 主题变量 → EP 组件 CSS（unplugin 自动注入在末尾）
// 若 EP 主题变量在 main.css 之前，Tailwind v4 重置可能覆盖 EP 主题变量导致按钮颜色异常
import './styles/main.css'
import 'element-plus/theme-chalk/dark/css-vars.css'
import * as ElementPlusIconsVue from '@element-plus/icons-vue'

import App from './App.vue'
import router, { restoreDynamicRoutes } from './router'
// ponytail: 国际化已迁回前端 ts,无需再 restore
import i18n, { restoreLocale } from './i18n'
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
// restoreI18n()

app.directive('auth', vAuth)
app.mount('#app')