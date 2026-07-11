<template>
  <div class="i18n-page h-full">
    <el-card class="h-full">
      <!-- ponytail: 带 total 提示的自定义搜索栏，绑定 useCrud.searchForm.key -->
      <div class="search-bar">
        <el-input
          v-model="searchForm.key"
          :placeholder="$t('common.search') + ' Key...'"
          clearable
          style="width: 260px"
          @keyup.enter="search"
        >
          <template #prefix><el-icon><Search /></el-icon></template>
        </el-input>
        <el-button type="primary" @click="search">{{ $t('common.search') }}</el-button>
        <el-button @click="reset">{{ $t('common.reset') }}</el-button>
        <span class="total-hint" style="margin-left: auto">{{ $t('sys.files.totalItems', { n: total }) }}</span>
      </div>

      <OnTable
        :data="rows"
        :columns="tableColumns"
        :loading="loading"
        :pagination="{ total, currentPage: page, pageSize }"
        :options="{ height: 'auto' }"
        @page-change="onPageChange"
        @command="handleCommand"
        @reload="load"
      >
        <template #toolbar-left>
          <el-button v-auth="'sys:i18n:add'" type="primary" @click="openAddKey">{{ $t('common.add') }}</el-button>
          <el-button v-auth="'sys:i18n:import'" type="primary" @click="openAddLocale">{{ $t('sys.rbac.addLocale') }}</el-button>
          <el-button v-auth="'sys:i18n:export'" type="success" @click="batchSave" :loading="saving">{{ $t('common.save') }}</el-button>
        </template>
        <!-- 每个 locale 一列可编辑输入 -->
        <template v-for="loc in locales" :key="loc" #[localeSlot(loc)]="{ row }">
          <el-input v-model="row.values[loc]" size="small" @input="row._dirty = true" />
        </template>
      </OnTable>
    </el-card>

    <OnDialog v-model="showAddLocale" :title="$t('sys.rbac.addLocale')" width="400px">
      <el-form label-width="80px">
        <el-form-item :label="$t('sys.rbac.locale')">
          <el-input v-model="newLocale" placeholder="ja-JP / ko-KR / fr-FR ..." />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showAddLocale = false">{{ $t('common.cancel') }}</el-button>
        <el-button type="primary" @click="addLocale">{{ $t('common.confirm') }}</el-button>
      </template>
    </OnDialog>

    <OnDialog v-model="showAddKey" :title="$t('common.add')" width="500px">
      <el-form label-width="100px">
        <el-form-item label="Key">
          <el-input v-model="newKey" placeholder="sys.menu.newItem" />
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
import { ref, reactive, computed, onMounted, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { Search } from '@element-plus/icons-vue'
import { useI18n } from 'vue-i18n'
import OnDialog from '@/components/OnDialog/index.vue'
import OnTable from '@/components/OnTable/index.vue'
import type { TableColumn } from '@/components/OnTable/types'
import { listI18nLocales, listI18n, upsertI18n, deleteI18n } from '@/api/sys/i18n'
import { useCrud, useMessage } from '@/hooks'

const { t } = useI18n()
const { success, error, confirm } = useMessage()

interface I18nRaw { id: number; locale: string; key: string; value: string }
interface Row { key: string; values: Record<string, string>; _dirty: boolean; _ids: Record<string, number> }

const locales = ref<string[]>([])
const rows = ref<Row[]>([])
const saving = ref(false)

const showAddLocale = ref(false)
const newLocale = ref('')
const showAddKey = ref(false)
const newKey = ref('')
const newValues = reactive<Record<string, string>>({})

// ponytail: listI18n 返回扁平 {id,locale,key,value}，前端聚合成每 key 一行；useCrud 负责分页/加载
const { loading, dataList, total, page, pageSize, searchForm, load, search, reset } = useCrud({
  getListApi: async (params?: any) => {
    if (!locales.value.length) locales.value = (await listI18nLocales()) || []
    return listI18n(params)
  },
  isPage: true,
  pageSize: 100,
  searchForm: { key: '' },
})

const localeSlot = (loc: string) => `loc_${loc}`

const tableColumns = computed<TableColumn[]>(() => [
  { prop: 'key', label: 'Key', minWidth: 220, fixed: 'left', sortable: true },
  ...locales.value.map((loc) => ({ prop: `loc_${loc}`, label: loc, minWidth: 200, slot: localeSlot(loc) })),
  {
    label: 'common.action',
    width: 80,
    fixed: 'right',
    buttons: [{ name: 'common.delete', command: 'delete', type: 'danger', size: 'small' }],
  },
])

// dataList → 聚合行；watch 覆盖 search/reset/load 所有加载路径
function rebuildRows() {
  const list = dataList.value as unknown as I18nRaw[]
  const map = new Map<string, Row>()
  for (const e of list) {
    if (!map.has(e.key)) map.set(e.key, { key: e.key, values: {}, _dirty: false, _ids: {} })
    const row = map.get(e.key)!
    row.values[e.locale] = e.value
    row._ids[e.locale] = e.id
  }
  rows.value = Array.from(map.values()).sort((a, b) => a.key.localeCompare(b.key))
}
watch(dataList, rebuildRows)

function onPageChange(p: number) { page.value = p; load() }

function handleCommand(command: string | number, row: Row) {
  if (command === 'delete') del(row)
}

function openAddLocale() { newLocale.value = ''; showAddLocale.value = true }

async function addLocale() {
  const loc = newLocale.value.trim()
  if (!loc) return
  if (!locales.value.includes(loc)) locales.value.push(loc)
  showAddLocale.value = false
  newLocale.value = ''
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
      await upsertI18n({ locale: loc, entries: [{ key: newKey.value.trim(), value: val }] })
    }
    ElMessage.success('ok')
    showAddKey.value = false
    await load()
  } catch (e: any) {
    error(e?.message || 'common.fail')
  } finally {
    saving.value = false
  }
}

async function batchSave() {
  const dirty = rows.value.filter((r) => r._dirty)
  if (!dirty.length) { ElMessage.info(t('sys.rbac.noChange')); return }
  saving.value = true
  try {
    for (const loc of locales.value) {
      const entries = dirty
        .filter((r) => r.values[loc] !== undefined && r.values[loc] !== '')
        .map((r) => ({ key: r.key, value: r.values[loc] }))
      if (!entries.length) continue
      await upsertI18n({ locale: loc, entries })
    }
    ElMessage.success(t('sys.rbac.savedN', { n: dirty.length }))
    dirty.forEach((r) => (r._dirty = false))
  } catch (e: any) {
    error(e?.message || 'common.fail')
  } finally {
    saving.value = false
  }
}

async function del(row: Row) {
  const ok = await confirm({ message: 'common.confirmDelete' })
  if (!ok) return
  try {
    for (const id of Object.values(row._ids)) await deleteI18n(id)
    success('common.success')
    await load()
  } catch (e: any) {
    error(e?.message || 'common.fail')
  }
}

onMounted(load)
</script>

<style scoped>
.search-bar { display: flex; gap: 12px; align-items: center; margin-bottom: 12px; }
.total-hint { font-size: 13px; color: var(--el-text-color-secondary); }
</style>
