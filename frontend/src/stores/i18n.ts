import { defineStore } from 'pinia'
import { ref } from 'vue'

export type LocaleMessages = Record<string, Record<string, string>>

export const useI18nStore = defineStore(
  'i18n',
  () => {
    const messages = ref<LocaleMessages>({})

    function setAll(data: LocaleMessages) {
      messages.value = data
    }

    function clear() {
      messages.value = {}
    }

    function isEmpty() {
      return Object.keys(messages.value).length === 0
    }

    return { messages, setAll, clear, isEmpty }
  },
  { persist: true }
)