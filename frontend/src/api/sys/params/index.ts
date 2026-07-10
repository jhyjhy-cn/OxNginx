import { getData, postData, putData, deleteData } from '@/api/http'
import type { PageQuery, PageResult } from '../users/type'
import type { ParamItem, ParamPayload } from './type'

export const pageParams = (params?: PageQuery) =>
  getData<PageResult<ParamItem>>('/api/rbac/params', params)

export const getParam = (id: number) => getData<ParamItem>(`/api/rbac/params/${id}`)

export const getParamByKey = (key: string) =>
  getData<ParamItem>(`/api/rbac/params/key/${key}`)

export const createParam = (payload: ParamPayload) =>
  postData<ParamItem>('/api/rbac/params', payload)

export const updateParam = (id: number, payload: ParamPayload) =>
  putData(`/api/rbac/params/${id}`, payload)

export const deleteParam = (id: number) => deleteData(`/api/rbac/params/${id}`)