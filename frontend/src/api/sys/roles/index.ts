import { getData, postData, putData, deleteData } from '@/api/http'
import type { PageQuery, PageResult } from '../users/type'
import type { Role, RolePayload } from './type'

export const listRoles = (params?: PageQuery) =>
  getData<PageResult<Role>>('/api/rbac/roles', params)

export const createRole = (payload: RolePayload) =>
  postData<Role>('/api/rbac/roles', payload)

export const updateRole = (id: number, payload: RolePayload) =>
  putData(`/api/rbac/roles/${id}`, payload)

export const deleteRole = (id: number) => deleteData(`/api/rbac/roles/${id}`)

export const batchDeleteRoles = (ids: number[]) =>
  postData(`/api/rbac/roles/batch-delete`, ids)

export const getRoleMenus = (id: number) =>
  getData<number[]>(`/api/rbac/roles/${id}/menus`)

export const setRoleMenus = (id: number, menu_ids: number[]) =>
  putData(`/api/rbac/roles/${id}/menus`, { menu_ids })