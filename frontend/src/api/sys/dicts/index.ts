import { getData, postData, putData, deleteData } from '@/api/http'
import type { Dict, DictItem } from './type'

export const listDicts = () => getData<Dict[]>('/api/rbac/dicts')

export const getDict = (id: number) => getData<Dict>(`/api/rbac/dicts/${id}`)

export const createDict = (payload: { name: string; code: string; description?: string }) =>
  postData<Dict>('/api/rbac/dicts', payload)

export const updateDict = (id: number, payload: { name: string; code: string; description?: string; status?: string }) =>
  putData(`/api/rbac/dicts/${id}`, payload)

export const deleteDict = (id: number) => deleteData(`/api/rbac/dicts/${id}`)

export const createDictItem = (dictId: number, payload: Partial<DictItem>) =>
  postData<DictItem>(`/api/rbac/dicts/${dictId}/items`, payload)

export const updateDictItem = (id: number, payload: Partial<DictItem>) =>
  putData(`/api/rbac/dict-items/${id}`, payload)

export const deleteDictItem = (id: number) => deleteData(`/api/rbac/dict-items/${id}`)