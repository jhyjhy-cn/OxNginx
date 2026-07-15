<template>
  <OnDialog
    v-model="visible"
    :title="db?.name || $t('dbm.manage')"
    width="90%"
    height="80vh"
    :maximizable="true"
    :close-on-click-modal="false"
  >
    <div v-if="!db" class="empty">{{ $t('dbm.selectTable') }}</div>
    <div v-else class="manage-root">
      <el-aside class="left" width="240px">
        <div class="left-header">
          <el-button type="primary" size="small" @click="openCreateTable">
            + {{ $t('dbm.createTable') }}
          </el-button>
          <span class="count">({{ tables.length }})</span>
        </div>
        <div class="left-list">
          <!-- 特殊项:新建查询 -->
          <div
            class="table-item table-item--virtual"
            :class="{ active: activeView === 'query' }"
            @click="selectQuery"
          >
            <div class="name">+ {{ $t('dbm.newQuery') }}</div>
            <div class="meta">{{ $t('dbm.run') }} SQL</div>
          </div>
          <!-- 表列表 -->
          <div
            v-for="t in tables"
            :key="t.name"
            class="table-item"
            :class="{ active: t.name === activeTable }"
            @click="selectTable(t.name)"
          >
            <div class="name" :title="t.name">{{ t.name }}</div>
            <div class="meta">
              {{ t.row_count }} {{ $t('dbm.rows') }}
            </div>
            <div class="actions">
              <el-button type="primary" link size="small" @click.stop="openRenameTable(t.name)">
                {{ $t('common.edit') }}
              </el-button>
              <el-button type="danger" link size="small" @click.stop="onDropTable(t.name)">
                {{ $t('common.delete') }}
              </el-button>
            </div>
          </div>
          <div v-if="!tables.length && !loading" class="empty">{{ $t('sys.files.emptyDir') }}</div>
        </div>
      </el-aside>

      <el-main class="right">
        <div class="toolbar">
          <span v-if="activeView === 'query'" class="active-table">{{ $t('dbm.newQuery') }}</span>
          <span v-else-if="activeTable" class="active-table">{{ activeTable }}</span>
          <span v-else class="muted">{{ $t('dbm.selectTable') }}</span>
        </div>

        <!-- 查询模式 -->
        <template v-if="activeView === 'query'">
          <div class="query-pane">
            <SqlEditor v-model="querySql" height="220px" />
            <div class="query-actions">
              <el-button type="primary" size="small" :loading="querying" @click="onRunQuery">
                {{ $t('dbm.run') }}
              </el-button>
              <el-button size="small" @click="querySql = ''">{{ $t('common.clear') }}</el-button>
            </div>
            <div v-if="queryError" class="query-error">{{ queryError }}</div>
            <div v-if="queryResult" class="query-result">
              <div class="muted">
                {{ $t('dbm.runResult') }}: {{ queryResult.rows.length }} {{ $t('dbm.rows') }}
              </div>
              <el-table :data="queryResult.rows" size="small" border max-height="360">
                <el-table-column
                  v-for="col in queryResult.columns"
                  :key="col"
                  :label="col"
                  :prop="col"
                  min-width="120"
                  show-overflow-tooltip
                >
                  <template #default="{ row }">
                    <span>{{ formatCell(row[col]) }}</span>
                  </template>
                </el-table-column>
              </el-table>
            </div>
          </div>
        </template>

        <!-- 表模式 -->
        <template v-else>

        <!-- 字段区 -->
        <div v-if="activeTable" class="cols-pane">
          <div class="pane-header">
            <span>{{ $t('dbm.cols') }} ({{ data?.columns?.length || 0 }})</span>
            <el-button type="primary" size="small" @click="openAddColumn">
              + {{ $t('dbm.addCol') }}
            </el-button>
          </div>
          <el-table
            v-if="data"
            :data="data.columns"
            size="small"
            border
            style="width: 100%"
            height="180"
          >
            <el-table-column prop="name" :label="$t('common.name')" min-width="120" />
            <el-table-column prop="type" :label="$t('common.type')" min-width="100" />
            <el-table-column :label="$t('dbm.notnull')" width="80">
              <template #default="{ row }">
                <el-tag v-if="row.notnull" type="warning" size="small">NOT NULL</el-tag>
                <span v-else>-</span>
              </template>
            </el-table-column>
            <el-table-column :label="$t('dbm.pk')" width="60">
              <template #default="{ row }">
                <el-tag v-if="row.pk" type="success" size="small">PK</el-tag>
                <span v-else>-</span>
              </template>
            </el-table-column>
            <el-table-column prop="default_value" label="default" min-width="100" show-overflow-tooltip />
            <el-table-column :label="$t('common.action')" width="160" fixed="right">
              <template #default="{ row }">
                <el-button type="primary" link size="small" @click="openRenameColumn(row.name)">
                  {{ $t('common.edit') }}
                </el-button>
                <el-button type="danger" link size="small" @click="onDropColumn(row.name)">
                  {{ $t('common.delete') }}
                </el-button>
              </template>
            </el-table-column>
          </el-table>
        </div>

        <!-- 数据表 -->
        <div v-if="activeTable" class="data-pane">
          <div class="pane-header">
            <span>{{ $t('dbm.rows') }} ({{ data?.total ?? 0 }})</span>
            <el-button
              type="primary"
              size="small"
              @click="openInsert"
              :disabled="!data"
            >
              + {{ $t('dbm.addRow') }}
            </el-button>
          </div>
          <el-table
            v-if="data"
            :data="data.rows"
            v-loading="loadingData"
            size="small"
            border
            style="width: 100%"
            height="280"
          >
            <el-table-column
              v-for="c in data.columns"
              :key="c.name"
              :label="c.name"
              :prop="c.name"
              min-width="120"
              show-overflow-tooltip
            >
              <template #default="{ row }">
                <span>{{ formatCell(row[c.name]) }}</span>
              </template>
            </el-table-column>
            <el-table-column :label="$t('common.action')" width="160" fixed="right">
              <template #default="{ row }">
                <el-button type="primary" link size="small" @click="openEdit(row)">
                  {{ $t('common.edit') }}
                </el-button>
                <el-button type="danger" link size="small" @click="onDeleteRow(row)">
                  {{ $t('common.delete') }}
                </el-button>
              </template>
            </el-table-column>
          </el-table>

          <div v-if="data && data.total > (data.page_size ?? 20)" class="pager">
            <el-pagination
              small
              layout="prev, pager, next, total"
              :total="data.total"
              :page-size="data.page_size"
              :current-page="data.page"
              @current-change="onPage"
            />
          </div>
        </div>
        </template>

        <!-- 新建查询已上移到顶部,旧折叠块删除 -->
      </el-main>
    </div>

    <!-- 行编辑 -->
    <OnDialog
      v-model="editVisible"
      :title="editMode === 'insert' ? $t('common.add') : $t('common.edit')"
      width="520px"
      append-to-body
    >
      <el-form :model="editForm" label-width="120px" v-loading="!editForm._ready">
        <el-form-item
          v-for="c in data?.columns || []"
          :key="c.name"
          :label="c.name"
        >
          <el-input-number
            v-if="editForm._ready && isNumberType(c.type)"
            v-model="editForm.values[c.name]"
            :controls="false"
            style="width: 100%"
          />
          <el-switch
            v-else-if="editForm._ready && isBoolType(c.type)"
            v-model="editForm.values[c.name]"
            :active-value="true"
            :inactive-value="false"
          />
          <el-input
            v-else-if="editForm._ready"
            v-model="editForm.values[c.name]"
            :placeholder="c.name"
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="editVisible = false">{{ $t('common.cancel') }}</el-button>
        <el-button type="primary" :loading="editSubmitting" @click="onSubmitEdit">
          {{ $t('common.confirm') }}
        </el-button>
      </template>
    </OnDialog>

    <!-- 通用确认弹窗(替代 ElMessageBox,避免被 OnDialog 容器污染) -->
    <OnDialog
      v-model="confirmVisible"
      :title="confirmState.title"
      width="420px"
      :maximizable="false"
      :close-on-click-modal="false"
    >
      <div class="confirm-body">{{ confirmState.message }}</div>
      <template #footer>
        <el-button @click="onConfirmCancel">{{ $t('common.cancel') }}</el-button>
        <el-button
          :type="confirmState.danger ? 'danger' : 'primary'"
          :loading="confirmState.loading"
          @click="onConfirmOk"
        >
          {{ confirmState.confirmText || $t('common.confirm') }}
        </el-button>
      </template>
    </OnDialog>

    <!-- 创建表 -->
    <OnDialog
      v-model="createTableVisible"
      :title="$t('dbm.createTable')"
      width="560px"
      append-to-body
    >
      <el-form label-width="100px">
        <el-form-item :label="$t('dbm.tableName')">
          <el-input v-model="newTableName" placeholder="users" />
        </el-form-item>
        <el-form-item :label="$t('dbm.cols')">
          <el-button size="small" @click="addColRow">+</el-button>
        </el-form-item>
        <div v-for="(c, i) in newTableCols" :key="i" class="col-row">
          <el-input v-model="c.name" placeholder="name" style="width: 30%" />
          <el-select v-model="c.type" style="width: 22%; margin: 0 8px" filterable allow-create
            default-first-option>
            <el-option-group
              v-for="g in columnTypeGroups"
              :key="g.label"
              :label="g.label"
            >
              <el-option v-for="t in g.types" :key="t" :label="t" :value="t" />
            </el-option-group>
          </el-select>
          <el-input v-model="c.extra" placeholder="NOT NULL DEFAULT 0" style="flex: 1" />
          <el-button link type="danger" size="small" @click="newTableCols.splice(i, 1)">×</el-button>
        </div>
        <el-form-item :label="$t('dbm.preview')">
          <pre class="preview">{{ previewCreateSql }}</pre>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="createTableVisible = false">{{ $t('common.cancel') }}</el-button>
        <el-button type="primary" :loading="createTableSubmitting" @click="onSubmitCreateTable">
          {{ $t('common.confirm') }}
        </el-button>
      </template>
    </OnDialog>

    <!-- 重命名表/列 -->
    <OnDialog
      v-model="renameVisible"
      :title="renameTarget.kind === 'table' ? $t('dbm.renameTable') : $t('dbm.renameColumn')"
      width="420px"
      append-to-body
    >
      <el-form label-width="120px">
        <el-form-item :label="$t('dbm.newName')">
          <el-input v-model="renameNew" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="renameVisible = false">{{ $t('common.cancel') }}</el-button>
        <el-button type="primary" :loading="renameSubmitting" @click="onSubmitRename">
          {{ $t('common.confirm') }}
        </el-button>
      </template>
    </OnDialog>

    <!-- 添加列 -->
    <OnDialog
      v-model="addColumnVisible"
      :title="$t('dbm.addCol')"
      width="480px"
      append-to-body
    >
      <el-form label-width="100px">
        <el-form-item :label="$t('common.name')">
          <el-input v-model="newColName" placeholder="email" />
        </el-form-item>
        <el-form-item :label="$t('common.type')">
          <el-select v-model="newColType" style="width: 100%" filterable allow-create
            default-first-option placeholder="选择或输入类型">
            <el-option-group
              v-for="g in columnTypeGroups"
              :key="g.label"
              :label="g.label"
            >
              <el-option v-for="t in g.types" :key="t" :label="t" :value="t" />
            </el-option-group>
          </el-select>
        </el-form-item>
        <el-form-item :label="$t('dbm.constraint')">
          <el-input v-model="newColExtra" placeholder="NOT NULL DEFAULT ''" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="addColumnVisible = false">{{ $t('common.cancel') }}</el-button>
        <el-button type="primary" :loading="addColumnSubmitting" @click="onSubmitAddColumn">
          {{ $t('common.confirm') }}
        </el-button>
      </template>
    </OnDialog>
  </OnDialog>
</template>

<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import OnDialog from '@/components/OnDialog/index.vue'
import SqlEditor from './SqlEditor.vue'
import { useMessage } from '@/hooks'
import type { DatabaseConn, TableInfo, TableData, SqlResult } from '@/api/databases/type'
import {
  listSqliteTables,
  sqliteTableData,
  sqliteRowInsert,
  sqliteRowUpdate,
  sqliteRowDelete,
  sqliteExec,
  sqliteCreateTable,
  sqliteRenameTable,
  sqliteDropTable,
  sqliteAddColumn,
  sqliteRenameColumn,
  sqliteDropColumn,
} from '@/api/databases'

const props = defineProps<{ visible: boolean; db: DatabaseConn | null }>()
const emit = defineEmits<{ 'update:visible': [v: boolean] }>()

const { t } = useI18n()
const { success, error, warning } = useMessage()
const visible = computed({ get: () => props.visible, set: (v) => emit('update:visible', v) })

const tables = ref<TableInfo[]>([])
const loading = ref(false)
const activeTable = ref('')
const activeView = ref<'table' | 'query'>('table')
const data = ref<TableData | null>(null)
const loadingData = ref(false)

const querySql = ref('')
const querying = ref(false)
const queryResult = ref<SqlResult | null>(null)
const queryError = ref<string | null>(null)

const dbPath = computed(() => props.db?.db_name || '')

// ========== 通用确认弹窗(替换 ElMessageBox) ==========
interface ConfirmState {
  title: string
  message: string
  danger?: boolean
  confirmText?: string
  loading?: boolean
  resolve?: (v: boolean) => void
}

// SQLite 字段类型分组(下拉显示,也可手输任意类型)
const columnTypeGroups: Array<{ label: string; types: string[] }> = [
  { label: '整数 (Integer)', types: ['INTEGER', 'INT', 'BIGINT', 'SMALLINT', 'TINYINT', 'MEDIUMINT'] },
  { label: '实数 (Real)', types: ['REAL', 'DOUBLE', 'FLOAT', 'NUMERIC', 'DECIMAL'] },
  { label: '文本 (Text)', types: ['TEXT', 'VARCHAR(255)', 'CHAR(36)', 'CLOB', 'NVARCHAR(255)'] },
  { label: '日期 (Date/Time)', types: ['DATE', 'DATETIME', 'TIMESTAMP', 'TIME'] },
  { label: '二进制 (Blob)', types: ['BLOB'] },
]
const confirmVisible = ref(false)
const confirmState = reactive<ConfirmState>({ title: '', message: '' })
function showConfirm(opts: {
  title?: string
  message: string
  danger?: boolean
  confirmText?: string
}): Promise<boolean> {
  confirmState.title = opts.title || t('common.tip')
  confirmState.message = opts.message
  confirmState.danger = opts.danger
  confirmState.confirmText = opts.confirmText
  confirmState.loading = false
  confirmVisible.value = true
  return new Promise((resolve) => {
    confirmState.resolve = resolve
  })
}
function onConfirmOk() {
  if (confirmState.resolve) {
    confirmState.resolve(true)
    confirmState.resolve = undefined
  }
  confirmVisible.value = false
}
function onConfirmCancel() {
  if (confirmState.resolve) {
    confirmState.resolve(false)
    confirmState.resolve = undefined
  }
  confirmVisible.value = false
}

async function resetState() {
  activeTable.value = ''
  data.value = null
  querySql.value = ''
  queryResult.value = null
  queryError.value = null
  await loadTables()
}

async function loadTables() {
  if (!dbPath.value) return
  loading.value = true
  try {
    tables.value = (await listSqliteTables(dbPath.value)) || []
  } catch (e: any) {
    // @ts-ignore
    error(e?.message || t('common.failed'))
    tables.value = []
  } finally {
    loading.value = false
  }
}

async function selectTable(name: string) {
  activeTable.value = name
  activeView.value = 'table'
  data.value = null
  await loadData(1)
}

function selectQuery() {
  activeView.value = 'query'
  activeTable.value = ''
  data.value = null
}

async function loadData(page: number) {
  if (!activeTable.value || !dbPath.value) return
  loadingData.value = true
  try {
    data.value = await sqliteTableData({
      path: dbPath.value,
      table: activeTable.value,
      page,
      page_size: 20,
    })
  } catch (e: any) {
    // @ts-ignore
    error(e?.message || t('common.failed'))
    data.value = null
  } finally {
    loadingData.value = false
  }
}

function onPage(p: number) {
  loadData(p)
}

function formatCell(v: any) {
  if (v === null || v === undefined) return ''
  if (typeof v === 'string') return v
  if (typeof v === 'object') return JSON.stringify(v)
  return String(v)
}

// 字段类型 → 输入控件分发
function isNumberType(t: string) {
  const u = t.toUpperCase()
  return u.includes('INT') || u.includes('REAL') || u.includes('FLOAT') || u.includes('DOUB') || u.includes('NUM') || u.includes('DEC')
}
function isBoolType(t: string) {
  const u = t.toUpperCase()
  return u.includes('BOOL')
}

// ========== 行编辑 ==========
const editVisible = ref(false)
const editMode = ref<'insert' | 'edit'>('insert')
const editForm = reactive<{ _ready: boolean; values: Record<string, any> }>({
  _ready: false,
  values: {},
})
const editSubmitting = ref(false)

function openInsert() {
  editMode.value = 'insert'
  const values: Record<string, any> = {}
  for (const c of data.value?.columns || []) {
    // ponytail: 数字/布尔列初值给 null,el-input-number/switch 不会因空串报错
    values[c.name] = isNumberType(c.type) || isBoolType(c.type) ? null : ''
  }
  Object.assign(editForm, { _ready: true, values })
  editVisible.value = true
}

function openEdit(row: any) {
  editMode.value = 'edit'
  const values: Record<string, any> = {}
  for (const c of data.value?.columns || []) {
    const raw = row[c.name]
    if (isNumberType(c.type)) {
      values[c.name] = raw === null || raw === undefined || raw === '' ? null : Number(raw)
    } else if (isBoolType(c.type)) {
      values[c.name] = Boolean(raw)
    } else {
      values[c.name] = raw ?? ''
    }
  }
  Object.assign(editForm, { _ready: true, values })
  editVisible.value = true
}

async function onSubmitEdit() {
  if (!data.value) return
  editSubmitting.value = true
  try {
    const pkSet = new Set(data.value.primary_key || [])
    const values: Record<string, any> = {}
    for (const c of data.value.columns) {
      const v = editForm.values[c.name]
      if (editMode.value === 'insert' && pkSet.has(c.name)) continue
      if (v === '' || v === null || v === undefined) continue
      values[c.name] = v
    }
    if (editMode.value === 'insert') {
      await sqliteRowInsert({ path: dbPath.value, table: activeTable.value, values })
    } else {
      const pk: Record<string, any> = {}
      for (const k of data.value.primary_key || []) pk[k] = editForm.values[k]
      await sqliteRowUpdate({ path: dbPath.value, table: activeTable.value, pk, values })
    }
    // @ts-ignore
    success(t('common.success'))
    editVisible.value = false
    await loadTables()
    await loadData(data.value?.page || 1)
  } catch (e: any) {
    // @ts-ignore
    error(e?.message || t('common.failed'))
  } finally {
    editSubmitting.value = false
  }
}

async function onDeleteRow(row: any) {
  if (!data.value) return
  const ok = await showConfirm({ message: t('dbm.confirmDeleteRow'), danger: true })
  if (!ok) return
  const pk: Record<string, any> = {}
  for (const k of data.value.primary_key || []) pk[k] = row[k]
  try {
    await sqliteRowDelete({ path: dbPath.value, table: activeTable.value, pk })
    // @ts-ignore
    success(t('common.success'))
    await loadTables()
    await loadData(data.value?.page || 1)
  } catch (e: any) {
    // @ts-ignore
    error(e?.message || t('common.failed'))
  }
}

// ========== 新建查询 ==========
async function onRunQuery() {
  if (!querySql.value.trim()) return
  querying.value = true
  queryError.value = null
  queryResult.value = null
  try {
    queryResult.value = await sqliteExec({ path: dbPath.value, sql: querySql.value })
    if (queryResult.value.error) queryError.value = queryResult.value.error
  } catch (e: any) {
    queryError.value = e?.message || t('common.failed')
  } finally {
    querying.value = false
  }
}

// ========== 表 DDL ==========
const createTableVisible = ref(false)
const newTableName = ref('')
const newTableCols = ref<{ name: string; type: string; extra: string }[]>([
  { name: 'id', type: 'INTEGER', extra: 'PRIMARY KEY AUTOINCREMENT' },
])
const createTableSubmitting = ref(false)

const previewCreateSql = computed(() => {
  if (!newTableName.value.trim() || !newTableCols.value.length) return ''
  const cols = newTableCols.value
    .filter((c) => c.name.trim())
    .map((c) => `  "${c.name}" ${c.type}${c.extra.trim() ? ' ' + c.extra.trim() : ''}`)
    .join(',\n')
  return `CREATE TABLE "${newTableName.value.trim()}" (\n${cols}\n);`
})

function openCreateTable() {
  newTableName.value = ''
  newTableCols.value = [{ name: 'id', type: 'INTEGER', extra: 'PRIMARY KEY AUTOINCREMENT' }]
  createTableVisible.value = true
}

function addColRow() {
  newTableCols.value.push({ name: '', type: 'TEXT', extra: '' })
}

async function onSubmitCreateTable() {
  if (!newTableName.value.trim()) {
    // @ts-ignore
    warning(t('dbm.required'))
    return
  }
  createTableSubmitting.value = true
  try {
    await sqliteCreateTable({ path: dbPath.value, sql: previewCreateSql.value })
    // @ts-ignore
    success(t('common.success'))
    createTableVisible.value = false
    await loadTables()
    selectTable(newTableName.value.trim())
  } catch (e: any) {
    // @ts-ignore
    error(e?.message || t('common.failed'))
  } finally {
    createTableSubmitting.value = false
  }
}

async function onDropTable(name: string) {
  const ok = await showConfirm({
    message: `${t('common.confirmDelete')}: ${name}`,
    danger: true,
  })
  if (!ok) return
  try {
    await sqliteDropTable({ path: dbPath.value, name })
    // @ts-ignore
    success(t('common.success'))
    if (activeTable.value === name) {
      activeTable.value = ''
      data.value = null
    }
    await loadTables()
  } catch (e: any) {
    // @ts-ignore
    error(e?.message || t('common.failed'))
  }
}

// ========== 重命名 ==========
const renameVisible = ref(false)
const renameSubmitting = ref(false)
const renameNew = ref('')
const renameTarget = ref<{ kind: 'table' | 'column'; old: string }>({ kind: 'table', old: '' })

function openRenameTable(name: string) {
  renameTarget.value = { kind: 'table', old: name }
  renameNew.value = name
  renameVisible.value = true
}

function openRenameColumn(name: string) {
  renameTarget.value = { kind: 'column', old: name }
  renameNew.value = name
  renameVisible.value = true
}

async function onSubmitRename() {
  if (!renameNew.value.trim() || renameNew.value === renameTarget.value.old) {
    renameVisible.value = false
    return
  }
  renameSubmitting.value = true
  try {
    if (renameTarget.value.kind === 'table') {
      await sqliteRenameTable({
        path: dbPath.value,
        old: renameTarget.value.old,
        new: renameNew.value.trim(),
      })
      if (activeTable.value === renameTarget.value.old) {
        activeTable.value = renameNew.value.trim()
      }
    } else {
      await sqliteRenameColumn({
        path: dbPath.value,
        table: activeTable.value,
        old: renameTarget.value.old,
        new: renameNew.value.trim(),
      })
    }
    // @ts-ignore
    success(t('common.success'))
    renameVisible.value = false
    await loadTables()
    if (renameTarget.value.kind === 'column' && activeTable.value) {
      await loadData(data.value?.page || 1)
    }
  } catch (e: any) {
    // @ts-ignore
    error(e?.message || t('common.failed'))
  } finally {
    renameSubmitting.value = false
  }
}

// ========== 字段 DDL ==========
const addColumnVisible = ref(false)
const newColName = ref('')
const newColType = ref('TEXT')
const newColExtra = ref('')
const addColumnSubmitting = ref(false)

function openAddColumn() {
  newColName.value = ''
  newColType.value = 'TEXT'
  newColExtra.value = ''
  addColumnVisible.value = true
}

async function onSubmitAddColumn() {
  if (!newColName.value.trim()) {
    // @ts-ignore
    warning(t('dbm.required'))
    return
  }
  addColumnSubmitting.value = true
  try {
    const def = `${newColName.value.trim()} ${newColType.value}${newColExtra.value.trim() ? ' ' + newColExtra.value.trim() : ''}`
    await sqliteAddColumn({ path: dbPath.value, table: activeTable.value, col_def: def })
    // @ts-ignore
    success(t('common.success'))
    addColumnVisible.value = false
    await loadData(data.value?.page || 1)
  } catch (e: any) {
    // @ts-ignore
    error(e?.message || t('common.failed'))
  } finally {
    addColumnSubmitting.value = false
  }
}

async function onDropColumn(name: string) {
  const ok = await showConfirm({
    message: `${t('common.confirmDelete')}: ${name}`,
    danger: true,
  })
  if (!ok) return
  try {
    await sqliteDropColumn({ path: dbPath.value, table: activeTable.value, col: name })
    // @ts-ignore
    success(t('common.success'))
    await loadData(data.value?.page || 1)
  } catch (e: any) {
    // @ts-ignore
    error(e?.message || t('common.failed'))
  }
}

watch(visible, (v) => {
  if (v) {
    resetState()
  } else {
    tables.value = []
    activeTable.value = ''
    data.value = null
  }
})
</script>

<style scoped>
.manage-root {
  display: flex;
  height: 70vh;
}
.left {
  border-right: 1px solid var(--el-border-color);
  display: flex;
  flex-direction: column;
}
.left-header {
  padding: 8px 12px;
  font-weight: 600;
  border-bottom: 1px solid var(--el-border-color);
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.left-list {
  flex: 1;
  overflow-y: auto;
}
.table-item {
  padding: 8px 12px;
  cursor: pointer;
  border-bottom: 1px solid var(--el-border-color-lighter);
}
.table-item:hover {
  background: var(--el-fill-color-light);
}
.table-item.active {
  background: var(--el-color-primary-light-9);
  color: var(--el-color-primary);
}
.table-item .name {
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.table-item .meta {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}
.table-item .actions {
  margin-top: 4px;
  display: none;
}
.table-item:hover .actions,
.table-item.active .actions {
  display: block;
}
.right {
  padding: 0 12px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
.toolbar {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 0;
  border-bottom: 1px solid var(--el-border-color-lighter);
}
.active-table {
  font-weight: 600;
}
.muted {
  color: var(--el-text-color-secondary);
}
.pane-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 0;
  font-size: 13px;
  font-weight: 500;
}
.cols-pane,
.data-pane,
.query-pane {
  margin-top: 8px;
}
.pager {
  margin-top: 8px;
  display: flex;
  justify-content: flex-end;
}
.query-actions {
  margin: 8px 0;
  display: flex;
  gap: 8px;
}
.query-error {
  color: var(--el-color-danger);
  margin-bottom: 8px;
  font-size: 12px;
}
.query-result {
  margin-top: 8px;
}
.empty {
  text-align: center;
  color: var(--el-text-color-secondary);
  padding: 16px;
}
.col-row {
  display: flex;
  align-items: center;
  margin-bottom: 8px;
}
.preview {
  background: var(--el-fill-color-light);
  padding: 8px;
  border-radius: 4px;
  font-size: 12px;
  margin: 0;
  white-space: pre-wrap;
  word-break: break-all;
}
</style>
