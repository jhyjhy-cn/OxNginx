import api from './index'

export interface ApiResponse<T = unknown> {
  code: number
  message: string
  data: T
}

export class ApiError extends Error {
  code: number
  constructor(code: number, message: string) {
    super(message)
    this.code = code
    this.name = 'ApiError'
  }
}

function unwrap<T>(resp: { data: ApiResponse<T> }): T {
  const body = resp.data
  if (body.code !== 0) throw new ApiError(body.code, body.message)
  return body.data
}

export const getData = <T>(url: string, params?: Record<string, unknown>) =>
  api.get<ApiResponse<T>>(url, { params }).then(unwrap<T>)

export const postData = <T>(url: string, body?: unknown) =>
  api.post<ApiResponse<T>>(url, body).then(unwrap<T>)

export const putData = <T>(url: string, body?: unknown) =>
  api.put<ApiResponse<T>>(url, body).then(unwrap<T>)

export const deleteData = <T = null>(url: string, body?: unknown) =>
  api.delete<ApiResponse<T>>(url, body ? { data: body } : undefined).then(unwrap<T>)

export const deleteDataWithParams = <T = null>(url: string, params?: Record<string, unknown>) =>
  api.delete<ApiResponse<T>>(url, { params }).then(unwrap<T>)

export const postForm = <T>(url: string, form: FormData) =>
  api
    .post<ApiResponse<T>>(url, form, {
      headers: { 'Content-Type': 'multipart/form-data' },
    })
    .then(unwrap<T>)

// 原包壳返回（少数端点需要 code/message，比如登录）
export const postRaw = <T>(url: string, body?: unknown) =>
  api.post<ApiResponse<T>>(url, body).then((r) => r.data)