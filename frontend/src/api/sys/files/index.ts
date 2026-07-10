import { getData, postForm, deleteData, postData } from '@/api/http'
import type { PageQuery, PageResult } from '../users/type'
import type { FileItem, UploadResult } from './type'

export const pageFiles = (params?: PageQuery) =>
  getData<PageResult<FileItem>>('/api/rbac/files/page', params)

export const getFile = (id: number) => getData<FileItem>(`/api/rbac/files/${id}`)

export const deleteFile = (id: number) => deleteData(`/api/rbac/files/${id}`)

export const batchDeleteFiles = (ids: number[]) =>
  postData('/api/rbac/files/batch-delete', { ids })

export const uploadFile = (form: FormData) =>
  postForm<UploadResult>('/api/rbac/files/upload', form)