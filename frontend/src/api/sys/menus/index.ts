import { getData, postData, putData, deleteData } from '@/api/http'
import type { MenuItem } from './type'

export const listMenus = (params?: Record<string, unknown>) =>
  getData<MenuItem[]>('/api/rbac/menus', params)

export const createMenu = (payload: Partial<MenuItem>) =>
  postData<MenuItem>('/api/rbac/menus', payload)

export const updateMenu = (id: number, payload: Partial<MenuItem>) =>
  putData(`/api/rbac/menus/${id}`, payload)

export const deleteMenu = (id: number) => deleteData(`/api/rbac/menus/${id}`)

export const batchDeleteMenus = (ids: number[]) =>
  postData('/api/rbac/menus/batch-delete', ids)