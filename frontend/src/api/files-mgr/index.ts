import { getData, postData, deleteData } from '@/api/http'
import type {
  FileEntry,
  ReadFileResult,
  CalcSizeResult,
  CompressParams,
  ExtractParams,
} from './type'

export const listFiles = (params: { path: string; show_hidden?: boolean }) =>
  postData<FileEntry[]>('/api/files/list', params)

export const listRoots = () => getData<string[]>('/api/files/roots')

export const readFile = (path: string) =>
  postData<ReadFileResult>('/api/files/read', { path })

export const writeFile = (path: string, content: string) =>
  postData('/api/files/write', { path, content })

export const mkdir = (path: string, name: string) =>
  postData('/api/files/mkdir', { path, name })

export const touch = (path: string, name: string) =>
  postData('/api/files/touch', { path, name })

export const rename = (path: string, new_name: string) =>
  postData('/api/files/rename', { path, new_name })

export const moveFile = (source: string, destination: string) =>
  postData('/api/files/move', { source, destination })

export const copyFile = (source: string, destination: string) =>
  postData('/api/files/copy', { source, destination })

export const deleteFileEntry = (path: string) =>
  deleteData('/api/files/delete', { path })

export const chmod = (path: string, mode: string) =>
  postData('/api/files/chmod', { path, mode })

export const compress = (params: CompressParams) =>
  postData('/api/files/compress', params)

export const extract = (params: ExtractParams) =>
  postData('/api/files/extract', params)

export const saveNote = (path: string, note: string) =>
  postData('/api/files/note', { path, note })

export const calcSize = (path: string) =>
  postData<CalcSizeResult>('/api/files/size', { path })

export const downloadFileUrl = (path: string) =>
  `/api/files/download?path=${encodeURIComponent(path)}`