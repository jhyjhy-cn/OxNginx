export interface FileEntry {
  name: string
  path: string
  is_dir: boolean
  size: number
  modified?: string
  permissions?: string
  [key: string]: unknown
}

export interface ReadFileResult {
  content: string
  is_binary?: boolean
  [key: string]: unknown
}

export interface CalcSizeResult {
  size: number
}

export interface CompressParams {
  paths: string[]
  destination: string
  format?: string
}

export interface ExtractParams {
  path: string
  destination?: string
}