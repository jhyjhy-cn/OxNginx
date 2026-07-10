import { getData, postData, putData, deleteData } from '@/api/http'
import { useAuthStore } from '@/stores/auth'
import type {
  Dept,
  Post,
  Role,
  UserItem,
  UserPayload,
  PageResult,
  PageQuery,
} from './type'

export const listUsers = (params?: PageQuery) =>
  getData<PageResult<UserItem>>('/api/rbac/users', params)

export const getUser = (id: number) => getData<UserItem>(`/api/rbac/users/${id}`)

export const createUser = (payload: UserPayload) =>
  postData<UserItem>('/api/rbac/users', payload)

export const updateUser = (id: number, payload: UserPayload) =>
  putData<UserItem>(`/api/rbac/users/${id}`, payload)

export const deleteUser = (id: number) => deleteData(`/api/rbac/users/${id}`)

export const resetPassword = (id: number, new_password: string) =>
  postData<string>(`/api/rbac/users/${id}/reset-password`, { new_password })

export const batchResetPassword = (ids: number[]) =>
  postData<string>('/api/rbac/users/batch/reset-password', { ids })

export const batchSetDisabled = (ids: number[], disabled: number) =>
  postData<string>('/api/rbac/users/batch/disabled', { ids, disabled })

export const exportUsers = (params?: PageQuery) => {
  const qs = new URLSearchParams(params as Record<string, string>).toString()
  const authStore = useAuthStore()
  return fetch('/api/rbac/users/export?' + qs, {
    headers: { Authorization: `Bearer ${authStore.token ?? ''}` },
  })
}

export const getDeptTree = () => getData<Dept[]>('/api/rbac/depts/tree')

export const listPosts = (params?: PageQuery) =>
  getData<PageResult<Post>>('/api/rbac/posts', params)

export const listRoles = (params?: PageQuery) =>
  getData<PageResult<Role>>('/api/rbac/roles', params)