<template>
  <div class="rbac-page">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>{{ $t('menu.rbacI18n') }}</span>
          <div class="toolbar">
            <el-button type="primary" @click="openAddKey">{{ $t('common.add') }}</el-button>
            <el-button type="primary" @click="openAddLocale">新增语言</el-button>
            <el-button type="success" @click="batchSave" :loading="saving">{{ $t('common.save') }}</el-button>
            <el-button @click="load">{{ $t('common.refresh') }}</el-button>
          </div>
        </div>
      </template>

      <!-- 搜索栏 -->
      <div class="search-bar">
        <el-input
          v-model="search"
          :placeholder="$t('common.search') + ' Key...'"
          clearable
          style="width: 300px"
          @input="onSearchInput"
          @clear="currentPage = 1; load()"
        >
          <template #prefix><el-icon><Search /></el-icon></template>
        </el-input>
        <span class="total-hint">共 {{ total }} 条</span>
      </div>

      <el-table :data="rows" v-loading="loading" @selection-change="onSelect" max-height="calc(100vh - 310px)">
        <el-table-column type="selection" width="48" />
        <el-table-column prop="key" label="Key" min-width="220" fixed sortable />
        <el-table-column
          v-for="loc in locales"
          :key="loc"
          :label="loc"
          min-width="200"
        >
          <template #default="{ row }">
            <el-input
              v-model="row.values[loc]"
              size="small"
              @input="row._dirty = true"
            />
          </template>
        </el-table-column>
        <el-table-column :label="$t('common.action')" width="80" fixed="right">
          <template #default="{ row }">
            <el-button type="danger" text size="small" @click="del(row)">{{ $t('common.delete') }}</el-button>
          </template>
        </el-table-column>
      </el-table>

      <!-- 分页 -->
      <div class="pagination-bar">
        <el-pagination
          v-model:current-page="currentPage"
          v-model:page-size="pageSize"
          :total="total"
          :page-sizes="[50, 100, 200]"
          layout="total, sizes, prev, pager, next"
          small
          @current-change="load"
          @size-change="currentPage = 1; load()"
        />
      </div>
    </el-card>

    <!-- 新增语言 -->
    <OnDialog v-model="showAddLocale" title="新增语言" width="400px">
      <el-form label-width="80px">
        <el-form-item label="Locale">
          <el-input v-model="newLocale" placeholder="ja-JP / ko-KR / fr-FR ..." />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showAddLocale = false">{{ $t('common.cancel') }}</el-button>
        <el-button type="primary" @click="addLocale">{{ $t('common.confirm') }}</el-button>
      </template>
    </OnDialog>

    <!-- 新增翻译 key -->
    <OnDialog v-model="showAddKey" :title="$t('common.add')" width="500px">
      <el-form label-width="100px">
        <el-form-item label="Key">
          <el-input v-model="newKey" placeholder="menu.newItem" />
        </el-form-item>
        <el-form-item v-for="loc in locales" :key="loc" :label="loc">
          <el-input v-model="newValues[loc]" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showAddKey = false">{{ $t('common.cancel') }}</el-button>
        <el-button type="primary" @click="addKey" :loading="saving">{{ $t('common.confirm') }}</el-button>
      </template>
    </OnDialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Search } from '@element-plus/icons-vue'
import api from '@/api'
import OnDialog from '@/components/OnDialog/index.vue'

interface I18nRaw { id: number; locale: string; key: string; value: string }
interface Row {
  key: string
  values: Record<string, string>
  _dirty: boolean
  _ids: Record<string, number>
}

const locales = ref<string[]>([])
const rows = ref<Row[]>([])
const total = ref(0)
const loading = ref(false)
const saving = ref(false)
const selectedKeys = ref<string[]>([])
const search = ref('')
const currentPage = ref(1)
const pageSize = ref(100)

const showAddLocale = ref(false)
const newLocale = ref('')
const showAddKey = ref(false)
const newKey = ref('')
const newValues = reactive<Record<string, string>>({})

// 搜索防抖
let searchTimer: ReturnType<typeof setTimeout> | null = null
function onSearchInput() {
  if (searchTimer) clearTimeout(searchTimer)
  searchTimer = setTimeout(() => { currentPage.value = 1; load() }, 300)
}

onMounted(load)

async function load() {
  loading.value = true
  try {
    // 1. locale 列表（只在 locales 为空时拉一次）
    if (!locales.value.length) {
      const { data: locRes } = await api.get('/api/rbac/i18n/locales')
      if (locRes.code === 0) locales.value = locRes.data || []
    }

    // 2. 分页查询
    const params: Record<string, string | number> = {
      page: currentPage.value,
      page_size: pageSize.value,
    }
    if (search.value) params.key = search.value
    const { data } = await api.get('/api/rbac/i18n', { params })
    if (data.code !== 0) return

    const list: I18nRaw[] = data.data.list || []
    total.value = data.data.total || 0

    // 3. 按 key 聚合成行（同 key 多 locale 合并到 values）
    const map = new Map<string, Row>()
    for (const e of list) {
      if (!map.has(e.key)) {
        map.set(e.key, { key: e.key, values: {}, _dirty: false, _ids: {} })
      }
      const row = map.get(e.key)!
      row.values[e.locale] = e.value
      row._ids[e.locale] = e.id
    }
    rows.value = Array.from(map.values()).sort((a, b) => a.key.localeCompare(b.key))
  } finally {
    loading.value = false
  }
}

function onSelect(selected: Row[]) {
  selectedKeys.value = selected.map(r => r.key)
}

function openAddLocale() {
  newLocale.value = ''
  showAddLocale.value = true
}

async function addLocale() {
  const loc = newLocale.value.trim()
  if (!loc) return
  if (!locales.value.includes(loc)) locales.value.push(loc)
  showAddLocale.value = false
  newLocale.value = ''
  // 新语言列立刻出现，已有行的值为空
}

function openAddKey() {
  newKey.value = ''
  for (const loc of locales.value) newValues[loc] = ''
  showAddKey.value = true
}

async function addKey() {
  if (!newKey.value.trim()) return
  saving.value = true
  try {
    for (const loc of locales.value) {
      const val = newValues[loc] || ''
      if (!val) continue
      await api.post('/api/rbac/i18n', {
        locale: loc,
        entries: [{ key: newKey.value.trim(), value: val }],
      })
    }
    ElMessage.success('ok')
    showAddKey.value = false
    await load()
  } finally {
    saving.value = false
  }
}

async function batchSave() {
  const dirty = rows.value.filter(r => r._dirty)
  if (!dirty.length) {
    ElMessage.info('无修改')
    return
  }
  saving.value = true
  try {
    for (const loc of locales.value) {
      const entries = dirty
        .filter(r => r.values[loc] !== undefined && r.values[loc] !== '')
        .map(r => ({ key: r.key, value: r.values[loc] }))
      if (!entries.length) continue
      await api.post('/api/rbac/i18n', { locale: loc, entries })
    }
    ElMessage.success(`已保存 ${dirty.length} 条`)
    dirty.forEach(r => (r._dirty = false))
  } finally {
    saving.value = false
  }
}

async function del(row: Row) {
  try {
    await ElMessageBox.confirm(`确定删除「${row.key}」（所有语言）?`, '提示', { type: 'warning' })
    for (const id of Object.values(row._ids)) {
      await api.delete(`/api/rbac/i18n/${id}`)
    }
    ElMessage.success('ok')
    await load()
  } catch {}
}
</script>

<style scoped>
.toolbar { display: flex; gap: 12px; }
.card-header { display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 8px; }
.search-bar { display: flex; align-items: center; gap: 12px; margin-bottom: 12px; }
.total-hint { font-size: 13px; color: var(--el-text-color-secondary); }
.pagination-bar { display: flex; justify-content: flex-end; margin-top: 12px; }
</style>