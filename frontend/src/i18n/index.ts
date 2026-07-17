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

export default i18n