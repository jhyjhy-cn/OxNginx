import { getCurrentInstance } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'

type MsgType = 'success' | 'warning' | 'error' | 'info'

function t(key: string): string {
  const instance = getCurrentInstance()
  if (instance?.proxy) {
    return instance.proxy.$t(key) as string
  }
  return key
}

function i18n(key: string, params?: Record<string, any>): string {
  const msg = t(key)
  if (params) {
    return msg.replace(/\{(\w+)\}/g, (_, k) => String(params[k] ?? `{${k}}`))
  }
  return msg
}

export function useMessage() {
  function message(opts: { message: string; type?: MsgType; duration?: number }) {
    ElMessage({
      message: opts.message,
      type: opts.type || 'info',
      duration: opts.duration ?? 3000,
    })
  }

  function success(message: string | (() => string), duration?: number) {
    const msg = typeof message === 'function' ? message() : i18n(message)
    ElMessage.success({ message: msg, duration })
  }

  function error(message: string | (() => string), duration?: number) {
    const msg = typeof message === 'function' ? message() : i18n(message)
    ElMessage.error({ message: msg, duration: duration ?? 5000 })
  }

  function warning(message: string | (() => string), duration?: number) {
    const msg = typeof message === 'function' ? message() : i18n(message)
    ElMessage.warning({ message: msg, duration })
  }

  function info(message: string | (() => string), duration?: number) {
    const msg = typeof message === 'function' ? message() : i18n(message)
    ElMessage.info({ message: msg, duration })
  }

  async function confirm(opts: { message: string; title?: string; type?: MsgType }): Promise<boolean> {
    const msg = i18n(opts.message)
    const title = opts.title ? i18n(opts.title) : t('common.tip')
    try {
      await ElMessageBox.confirm(msg, title, { type: opts.type || 'warning' })
      return true
    } catch {
      return false
    }
  }

  async function prompt(opts: { message: string; title?: string; inputValue?: string }): Promise<string | false> {
    const msg = i18n(opts.message)
    const title = opts.title ? i18n(opts.title) : t('common.tip')
    try {
      const { value } = await ElMessageBox.prompt(msg, title, { inputValue: opts.inputValue })
      return value as string
    } catch {
      return false
    }
  }

  return { message, success, error, warning, info, confirm, prompt, t }
}
