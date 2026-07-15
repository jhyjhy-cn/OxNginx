import { useI18n } from 'vue-i18n'
import { ElMessage, ElMessageBox } from 'element-plus'

type MsgType = 'success' | 'warning' | 'error' | 'info'

export function useMessage() {
  const { t } = useI18n()

  function i18n(key: string, params?: Record<string, any>): string {
    const msg = t(key)
    const s = typeof msg === 'string' ? msg : String(msg)
    if (params) {
      return s.replace(/\{(\w+)\}/g, (_, k) => String(params[k] ?? `{${k}}`))
    }
    return s
  }
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

  async function confirm(opts: {
    message: string
    title?: string
    type?: MsgType
    params?: Record<string, any>
    confirmText?: string
    cancelText?: string
    checkbox?: { label: string; checked?: boolean }
  }): Promise<{ ok: boolean; checkboxChecked: boolean }> {
    const msg = i18n(opts.message, opts.params)
    const title = opts.title ? i18n(opts.title) : t('common.tip')
    // ponytail: el-message-box showCheckbox 仅在 2.13+ 运行时支持,types 还未声明
    const options: any = { type: opts.type || 'warning' }
    if (opts.confirmText) options.confirmButtonText = i18n(opts.confirmText)
    if (opts.cancelText) options.cancelButtonText = i18n(opts.cancelText)
    if (opts.checkbox) {
      options.showCheckbox = true
      options.checkboxLabel = i18n(opts.checkbox.label)
      options.checkboxChecked = !!opts.checkbox.checked
    }
    try {
      const res = await ElMessageBox.confirm(msg, title, options)
      return { ok: true, checkboxChecked: (res as any)?.checkboxChecked === true }
    } catch {
      return { ok: false, checkboxChecked: false }
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
