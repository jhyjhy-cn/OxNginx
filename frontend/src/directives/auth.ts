import type { Directive, DirectiveBinding } from 'vue'
import { useAuthStore } from '@/stores/auth'

/**
 * v-auth="'sys:user:add'"
 * v-auth="['sys:user:add', 'sys:user:delete']"  // 多选一，OR 逻辑
 *
 * - 单个字符串：无权限时移除元素
 * - 空数组/null/undefined：直接显示（不做权限校验）
 * - super_admin (admin) 永远放行
 */
function checkAuth(el: HTMLElement, binding: DirectiveBinding) {
  const authStore = useAuthStore()
  const value = binding.value

  // 空值/null/undefined：不校验，直接显示
  if (value === null || value === undefined || value === '') {
    return
  }

  let allowed = false
  if (typeof value === 'string') {
    allowed = authStore.hasPermission(value)
  } else if (Array.isArray(value) && value.length > 0) {
    // 多选一 OR
    allowed = value.some((code) => authStore.hasPermission(code as string))
  }

  if (!allowed) {
    el.remove()
  }
}

export const vAuth: Directive = {
  mounted(el: HTMLElement, binding: DirectiveBinding) {
    checkAuth(el, binding)
  },
  updated(el: HTMLElement, binding: DirectiveBinding) {
    checkAuth(el, binding)
  },
}
