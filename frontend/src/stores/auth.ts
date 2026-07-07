import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import api from '@/api'
import { mergeI18nMessages } from '@/i18n'

export interface MenuNode {
  id: number
  parent_id: number | null
  name: string
  title: string
  icon: string | null
  path: string | null
  component: string | null
  type: 'M' | 'C' | 'F'
  permission: string | null
  sort: number
  children: MenuNode[]
}

const LS = {
  token: 'token',
  username: 'username',
  roles: 'rbac_roles',
  perms: 'rbac_perms',
  menus: 'rbac_menus',
}

function loadJSON<T>(key: string, fallback: T): T {
  try {
    const raw = localStorage.getItem(key)
    return raw ? (JSON.parse(raw) as T) : fallback
  } catch {
    return fallback
  }
}

export const useAuthStore = defineStore('auth', () => {
  const token = ref<string>(localStorage.getItem(LS.token) || '')
  const username = ref<string>(localStorage.getItem(LS.username) || '')
  const roles = ref<string[]>(loadJSON<string[]>(LS.roles, []))
  const permissions = ref<string[]>(loadJSON<string[]>(LS.perms, []))
  const menus = ref<MenuNode[]>(loadJSON<MenuNode[]>(LS.menus, []))

  const isAuthenticated = computed(() => !!token.value)
  // ponytail: super_admin 硬编码短路；前端只走 username==='admin' 一条线
  const isSuperAdmin = computed(
    () => username.value === 'admin' || roles.value.includes('super_admin'),
  )
  const permissionSet = computed(() => new Set(permissions.value))

  function hasPermission(code: string) {
    return isSuperAdmin.value || permissionSet.value.has(code)
  }

  function persistRbac() {
    localStorage.setItem(LS.roles, JSON.stringify(roles.value))
    localStorage.setItem(LS.perms, JSON.stringify(permissions.value))
    localStorage.setItem(LS.menus, JSON.stringify(menus.value))
  }

  async function login(usernameInput: string, password: string) {
    const response = await api.post('/api/login', {
      username: usernameInput,
      password,
    })

    if (response.data.code === 0) {
      const d = response.data.data
      token.value = d.token
      username.value = d.username
      roles.value = d.roles ?? []
      permissions.value = d.permissions ?? []
      menus.value = d.menus ?? []
      localStorage.setItem(LS.token, token.value)
      localStorage.setItem(LS.username, username.value)
      persistRbac()
      // ponytail: 登录成功立即合并 DB 翻译,避免 Dashboard 首帧缺 key
      await fetchI18n()
      return true
    }

    throw new Error(response.data.message)
  }

  async function fetchRbacInfo() {
    // ponytail: 登录时已带 RBAC 信息；token 存在但 store 为空时刷新用
    if (!token.value) return
    try {
      const { data } = await api.get('/api/rbac/me')
      if (data.code === 0) {
        roles.value = data.data.roles ?? []
        permissions.value = data.data.permissions ?? []
        menus.value = data.data.menus ?? []
        persistRbac()
      }
    } catch {
      // 401 走拦截器；其他忽略
    }
  }

  async function fetchI18n() {
    // 从 DB 拉全量翻译（所有语言），合并到 vue-i18n
    if (!token.value) return
    try {
      const { data } = await api.get('/api/rbac/i18n')  // 不传 locale，返回全量
      if (data.code !== 0 || !data.data) return
      // 按 locale 分组
      const grouped: Record<string, Record<string, string>> = {}
      for (const e of data.data as { locale: string; key: string; value: string }[]) {
        if (!grouped[e.locale]) grouped[e.locale] = {}
        grouped[e.locale][e.key] = e.value
      }
      // 逐语言合并
      for (const [locale, flat] of Object.entries(grouped)) {
        mergeI18nMessages(locale, flat)
      }
    } catch {}
  }

  function logout() {
    token.value = ''
    username.value = ''
    roles.value = []
    permissions.value = []
    menus.value = []
    localStorage.removeItem(LS.token)
    localStorage.removeItem(LS.username)
    localStorage.removeItem(LS.roles)
    localStorage.removeItem(LS.perms)
    localStorage.removeItem(LS.menus)
  }

  function updateUser(newToken: string, newUsername: string) {
    token.value = newToken
    username.value = newUsername
    localStorage.setItem(LS.token, newToken)
    localStorage.setItem(LS.username, newUsername)
  }

  return {
    token,
    username,
    roles,
    permissions,
    menus,
    isAuthenticated,
    isSuperAdmin,
    permissionSet,
    hasPermission,
    login,
    logout,
    updateUser,
    fetchRbacInfo,
    fetchI18n,
    persistRbac,
  }
})