export interface FileItem {
  id: number
  name: string
  original_name: string
  suffix: string
  size: number
  mime_type?: string
  md5?: string
  path: string
  provider: string
  url: string
  created_at?: string
  uploaded_by?: string
}

export interface UploadResult {
  id: number
  name: string
  original_name: string
  path: string
  url: string
  size: number
  mime_type?: string
}