import { getData, postData, putData, deleteData, deleteDataWithParams } from '@/api/http'
import type {
  CreateDatabasePayload,
  DatabaseConn,
  DbTestResult,
  SqlResult,
  TableData,
  TableInfo,
  UpdateDatabasePayload,
} from './type'

export const listDatabases = (type?: string) =>
  getData<DatabaseConn[]>('/api/databases', type ? { type } : undefined)

export const getDatabase = (id: number) => getData<DatabaseConn>(`/api/databases/${id}`)

export const createDatabase = (payload: CreateDatabasePayload) =>
  postData<DatabaseConn>('/api/databases', payload)

export const updateDatabase = (id: number, payload: UpdateDatabasePayload) =>
  putData<DatabaseConn>(`/api/databases/${id}`, payload)

export const deleteDatabase = (id: number, removeFile = false) =>
  removeFile
    ? deleteDataWithParams(`/api/databases/${id}`, { remove_file: 'true' })
    : deleteData(`/api/databases/${id}`)

export const toggleDatabase = (id: number) =>
  postData<DatabaseConn>(`/api/databases/${id}/toggle`)

export const testDatabase = (id: number) =>
  postData<DbTestResult>(`/api/databases/${id}/test`)

// SQLite 管理
export const listSqliteTables = (path: string) =>
  postData<TableInfo[]>('/api/databases/sqlite/list-tables', { path })

export const sqliteTableData = (body: { path: string; table: string; page: number; page_size: number }) =>
  postData<TableData>('/api/databases/sqlite/table-data', body)

export const sqliteRowInsert = (body: { path: string; table: string; values: Record<string, unknown> }) =>
  postData<{ id: number }>('/api/databases/sqlite/row-insert', body)

export const sqliteRowUpdate = (body: {
  path: string
  table: string
  pk: Record<string, unknown>
  values: Record<string, unknown>
}) => postData<{ affected: number }>('/api/databases/sqlite/row-update', body)

export const sqliteRowDelete = (body: { path: string; table: string; pk: Record<string, unknown> }) =>
  postData<{ affected: number }>('/api/databases/sqlite/row-delete', body)

export const sqliteExec = (body: { path: string; sql: string }) =>
  postData<SqlResult>('/api/databases/sqlite/exec', body)

export const sqliteCreateTable = (body: { path: string; sql: string }) =>
  postData<string>('/api/databases/sqlite/create-table', body)

export const sqliteRenameTable = (body: { path: string; old: string; new: string }) =>
  postData<string>('/api/databases/sqlite/rename-table', body)

export const sqliteDropTable = (body: { path: string; name: string }) =>
  postData<string>('/api/databases/sqlite/drop-table', body)

export const sqliteAddColumn = (body: { path: string; table: string; col_def: string }) =>
  postData<string>('/api/databases/sqlite/add-column', body)

export const sqliteRenameColumn = (body: { path: string; table: string; old: string; new: string }) =>
  postData<string>('/api/databases/sqlite/rename-column', body)

export const sqliteDropColumn = (body: { path: string; table: string; col: string }) =>
  postData<string>('/api/databases/sqlite/drop-column', body)
