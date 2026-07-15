export interface DatabaseConn {
  id: number
  type: 'redis' | 'sqlite' | string
  name: string
  host?: string | null
  port?: number | null
  username?: string | null
  password?: string | null
  db_name?: string | null
  db_path?: string | null
  enabled: number
  sort?: number
  remark?: string | null
  version?: number
  created_by?: number | null
  updated_by?: number | null
  created_at?: string | null
  updated_at?: string | null
  _size_bytes?: number
  [key: string]: unknown
}

export interface DbTestResult {
  running: boolean
  not_installed: boolean
  version: string | null
  latency_ms: number | null
  error: string | null
}

export interface CreateDatabasePayload {
  type: string
  name: string
  host?: string | null
  port?: number | null
  username?: string | null
  password?: string | null
  db_name?: string | null
  db_path?: string | null
  enabled?: boolean
  sort?: number
  remark?: string | null
}

export type UpdateDatabasePayload = Partial<CreateDatabasePayload>

// SQLite 管理
export interface ColumnInfo {
  name: string
  type: string
  notnull: boolean
  pk: boolean
  default_value: string | null
}

export interface TableInfo {
  name: string
  row_count: number
  size_bytes: number
}

export interface TableData {
  columns: ColumnInfo[]
  primary_key: string[]
  rows: Array<Record<string, unknown>>
  total: number
  page: number
  page_size: number
}

export interface SqlResult {
  columns: string[]
  rows: Array<Record<string, unknown>>
  rows_affected: number
  error: string | null
}
