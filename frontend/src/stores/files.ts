import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useFilesStore = defineStore('files', () => {
  const lastPath = ref('')

  return { lastPath }
}, {
  persist: true,
})
