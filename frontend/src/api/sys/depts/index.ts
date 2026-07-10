import { getData, postData, putData, deleteData } from '@/api/http'
import type { Dept } from './type'
import type { PageQuery, PageResult } from '@/api/sys/users/type'

export const listDepts = (params?: PageQuery) =>
  getData<PageResult<Dept>>('/api/rbac/depts', params)

export const listDeptTree = () => getData<Dept[]>('/api/rbac/depts/tree')

export const createDept = (payload: Partial<Dept>) =>
  postData<Dept>('/api/rbac/depts', payload)

export const updateDept = (id: number, payload: Partial<Dept>) =>
  putData(`/api/rbac/depts/${id}`, payload)

export const deleteDept = (id: number) => deleteData(`/api/rbac/depts/${id}`)

export const batchDeleteDepts = (ids: number[]) =>
  postData(`/api/rbac/depts/batch-delete`, ids)