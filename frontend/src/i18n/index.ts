import { createI18n } from 'vue-i18n'
import zhCN from './locales/zh-CN'
import enUS from './locales/en-US'
import { useSettingsStore } from '@/stores/settings'

const i18n = createI18n({
  legacy: false,
  locale: 'zh-CN',
  fallbackLocale: 'zh-CN',
  messages: {
    'zh-CN': zhCN,
    'en-US': enUS,
  },
})

/** 从 settings store 恢复 locale（应用启动时调用） */
export function restoreLocale() {
  try {
    const settings = useSettingsStore()
    if (settings.locale && settings.locale !== i18n.global.locale.value) {
      i18n.global.locale.value = settings.locale
    }
  } catch {}
}

/**
 * 将 DB 下发的扁平 key-value 合并到 vue-i18n
 * 支持 'sys.menu.dashboard' -> { menu: { dashboard: '...' } } 嵌套结构
 * DB 翻译优先，静态文件作 fallback
 */
export function mergeI18nMessages(locale: string, flat: Record<string, string>) {
  const nested: Record<string, any> = {}
  for (const [key, value] of Object.entries(flat)) {
    const parts = key.split('.')
    let cur = nested
    for (let i = 0; i < parts.length - 1; i++) {
      cur[parts[i]] = cur[parts[i]] || {}
      cur = cur[parts[i]]
    }
    cur[parts[parts.length - 1]] = value
  }
  // 深度合并: DB 覆盖静态
  const existing = i18n.global.getLocaleMessage(locale) as Record<string, any>
  i18n.global.setLocaleMessage(locale, deepMerge(existing, nested))
}

function deepMerge(target: any, source: any): any {
  const result = { ...target }
  for (const key of Object.keys(source)) {
    if (source[key] && typeof source[key] === 'object' && !Array.isArray(source[key])) {
      result[key] = deepMerge(result[key] || {}, source[key])
    } else {
      result[key] = source[key]
    }
  }
  return result
}

export default i18n
