import type { App, Plugin } from 'vue'
import OnIcon from './index.vue'

const OnIconPlugin: Plugin = {
  install(app: App) {
    app.component('OnIcon', OnIcon)
  },
}

export { OnIcon, OnIconPlugin }
export default OnIconPlugin
