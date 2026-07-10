<template>
  <div class="rbac-page">
    <el-card>
      <div class="search-bar">
        <el-input
          v-model="keyword"
          :placeholder="$t('common.search')"
          clearable
          style="width: 240px"
          @input="onInput"
          @keyup.enter="doSearch"
        />
        <el-button type="primary" @click="doSearch">{{ $t('common.search') }}</el-button>
        <el-button @click="doReset">{{ $t('common.reset') }}</el-button>
      </div>

      <div class="toolbar">
        <el-button type="primary" @click="openCreate(null)">
          <el-icon><Plus /></el-icon>
          {{ $t('common.add') }}
        </el-button>
        <el-button type="danger" :disabled="!selectedIds.length" @click="batchDelete">
          <el-icon><Delete /></el-icon>
          {{ $t('sys.rbac.batchDelete') }} ({{ selectedIds.length }})
        </el-button>
        <el-button @click="load">{{ $t('common.refresh') }}</el-button>
        <span class="hint" style="margin-left: auto">{{ $t('sys.rbac.hintMenuRefresh') }}</span>
      </div>

      <el-table
        :data="menus"
        v-loading="loading"
        row-key="id"
        :tree-props="{ children: 'children' }"
        @selection-change="onSelect"
        ref="tableRef"
        max-height="calc(100vh - 380px)"
      >
        <el-table-column type="selection" width="48" />
        <el-table-column prop="name" :label="$t('sys.rbac.colName')" min-width="160">
          <template #default="{ row }">
            <el-icon v-if="row.icon" style="margin-right: 4px; vertical-align: middle"><component :is="row.icon" /></el-icon>
            <span>{{ row.name }}</span>
          </template>
        </el-table-column>
        <el-table-column prop="title" :label="$t('sys.rbac.colTitle')" min-width="160">
          <template #default="{ row }">
            <span>{{ $t(row.title) }}</span>
          </template>
        </el-table-column>
        <el-table-column prop="path" :label="$t('sys.rbac.colPath')" min-width="180" />
        <el-table-column prop="component" :label="$t('sys.rbac.colComponent')" min-width="140" />
        <el-table-column prop="type" :label="$t('sys.rbac.colType')" width="80">
          <template #default="{ row }">
            <el-tag size="small" :type="typeColor(row.type)">
              {{ $t(row.type === 'M' ? 'sys.rbac.typeM' : row.type === 'C' ? 'sys.rbac.typeC' : 'sys.rbac.typeF') }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="permission" :label="$t('sys.rbac.colPermission')" min-width="180" />
        <el-table-column prop="sort" :label="$t('sys.rbac.colSort')" width="80" />
        <el-table-column :label="$t('common.action')" width="200" fixed="right">
          <template #default="{ row }">
            <el-button v-if="row.type !== 'F'" type="primary" text size="small" @click="openCreate(row)">
              +{{ $t('sys.rbac.subItem') }}
            </el-button>
            <el-button type="primary" text size="small" @click="openEdit(row)">
              {{ $t('common.edit') }}
            </el-button>
            <el-button type="danger" text size="small" @click="del(row)">
              {{ $t('common.delete') }}
            </el-button>
          </template>
        </el-table-column>
      </el-table>

      <OnPagination
        v-model:current-page="currentPage"
        v-model:page-size="pageSize"
        :total="total"
        :page-sizes="[50, 100, 200]"
        @change="load"
      />
    </el-card>

    <OnDialog v-model="dialogVisible" :title="form.id ? $t('common.edit') : $t('common.add')" width="600px">
      <el-form :model="form" label-width="120px" :rules="rules" ref="formRef">
        <el-form-item :label="$t('sys.rbac.colType')" prop="type">
          <el-radio-group v-model="form.type">
            <el-radio-button value="M">{{ $t('sys.rbac.typeM') }}</el-radio-button>
            <el-radio-button value="C">{{ $t('sys.rbac.typeC') }}</el-radio-button>
            <el-radio-button value="F">{{ $t('sys.rbac.typeF') }}</el-radio-button>
          </el-radio-group>
        </el-form-item>
        <el-form-item :label="$t('sys.rbac.colParent')" prop="parent_id">
          <el-tree-select
            v-model="form.parent_id"
            :data="parentOptions"
            :props="{ label: 'name', value: 'id', children: 'children' }"
            check-strictly
            clearable
            :placeholder="$t('common.none')"
            style="width: 100%"
          />
        </el-form-item>
        <el-form-item :label="$t('sys.rbac.colName')" prop="name">
          <el-input v-model="form.name" />
        </el-form-item>
        <el-form-item :label="$t('sys.rbac.colTitle') + ' (i18n)'" prop="title">
          <el-input v-model="form.title" placeholder="sys.menu.sites" />
        </el-form-item>
        <el-form-item :label="$t('sys.rbac.colPath')" v-if="form.type !== 'F'">
          <el-input v-model="form.path" placeholder="/sites" />
        </el-form-item>
        <el-form-item :label="$t('sys.rbac.colComponent')" v-if="form.type === 'C'">
          <el-input v-model="form.component" placeholder="sites/index" />
        </el-form-item>
        <el-form-item :label="$t('sys.rbac.colPermission')">
          <el-input v-model="form.permission" placeholder="sys:site:view" />
        </el-form-item>
        <el-form-item :label="$t('sys.rbac.colIcon')" v-if="form.type !== 'F'">
          <el-input v-model="form.icon" placeholder="House / Connection / Lock..." />
        </el-form-item>
        <el-form-item :label="$t('sys.rbac.colSort')">
          <el-input-number v-model="form.sort" :min="0" :max="9999" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">{{ $t('common.cancel') }}</el-button>
        <el-button type="primary" @click="submit" :loading="submitting">{{ $t('common.confirm') }}</el-button>
      </template>
    </OnDialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, nextTick } from 'vue'
import { ElMessage } from 'element-plus'
import { Plus, Delete } from '@element-plus/icons-vue'
import { useI18n } from 'vue-i18n'
import OnPagination from '@/components/OnPagination/index.vue'
import OnDialog from '@/components/OnDialog/index.vue'
import {
  listMenus,
  createMenu,
  updateMenu,
  deleteMenu,
  batchDeleteMenus,
} from '@/api/sys/menus'
import type { MenuItem } from '@/api/sys/menus/type'
import { useMessage } from '@/hooks'

const { t } = useI18n()
const { success, error, confirm } = useMessage()

type Menu = MenuItem

const allMenus = ref<Menu[]>([])
const menus = ref<Menu[]>([])
const loading = ref(false)
const tableRef = ref()
const selectedIds = ref<number[]>([])
const keyword = ref('')
const currentPage = ref(1)
const pageSize = ref(100)
const total = ref(0)

const parentOptions = computed(() => {
  const filter = (nodes: Menu[]): Menu[] =>
    nodes.filter((n) => n.type !== 'F').map((n) => ({ ...n, children: n.children ? filter(n.children) : [] }))
  return filter(allMenus.value)
})

const dialogVisible = ref(false)
const submitting = ref(false)
const formRef = ref()
const form = reactive({
  id: null as number | null,
  type: 'C' as 'M' | 'C' | 'F',
  parent_id: null as number | null,
  name: '',
  title: '',
  path: '',
  component: '',
  permission: '',
  icon: '',
  sort: 0,
})

const rules = {
  name: [{ required: true, message: t('sys.rbac.required'), trigger: 'blur' }],
  title: [{ required: true, message: t('sys.rbac.required'), trigger: 'blur' }],
  type: [{ required: true, message: t('sys.rbac.required'), trigger: 'change' }],
}

function typeColor(t: string) {
  return t === 'M' ? 'success' : t === 'C' ? undefined : 'info'
}

function doSearch() {
  currentPage.value = 1
  load()
}
function doReset() {
  keyword.value = ''
  currentPage.value = 1
  load()
}

let timer: ReturnType<typeof setTimeout> | null = null
function onInput() {
  if (timer) clearTimeout(timer)
  timer = setTimeout(doSearch, 300)
}

onMounted(load)

async function load() {
  loading.value = true
  try {
    const params: Record<string, unknown> = { page: currentPage.value, page_size: pageSize.value }
    if (keyword.value) params.keyword = keyword.value
    const list: Menu[] = (await listMenus(params)) || []
    total.value = list.length
    const map = new Map<number, Menu>()
    list.forEach((m) => map.set(m.id, { ...m, children: [] }))
    const roots: Menu[] = []
    for (const m of map.values()) {
      if (m.parent_id && map.has(m.parent_id)) {
        map.get(m.parent_id)!.children!.push(m)
      } else {
        roots.push(m)
      }
    }
    allMenus.value = list
    menus.value = keyword.value ? list : roots
  } catch (e: any) {
    error(e?.message || "common.fail")
  } finally {
    loading.value = false
  }
}

function onSelect(rows: Menu[]) {
  selectedIds.value = collectIds(rows)
}

function collectIds(rows: Menu[]): number[] {
  const out: number[] = []
  const walk = (ns: Menu[]) => {
    for (const n of ns) {
      out.push(n.id)
      if (n.children?.length) walk(n.children)
    }
  }
  walk(rows)
  return out
}

function openCreate(parent: Menu | null) {
  form.id = null
  form.type = 'C'
  form.parent_id = parent?.id ?? null
  form.name = ''
  form.title = ''
  form.path = ''
  form.component = ''
  form.permission = ''
  form.icon = ''
  form.sort = 0
  dialogVisible.value = true
  nextTick(() => formRef.value?.clearValidate())
}

function openEdit(row: Menu) {
  form.id = row.id
  form.type = row.type
  form.parent_id = row.parent_id
  form.name = row.name
  form.title = row.title
  form.path = row.path ?? ''
  form.component = row.component ?? ''
  form.permission = row.permission ?? ''
  form.icon = row.icon ?? ''
  form.sort = row.sort
  dialogVisible.value = true
  nextTick(() => formRef.value?.clearValidate())
}

async function submit() {
  const valid = await formRef.value?.validate().catch(() => false)
  if (!valid) return
  submitting.value = true
  try {
    const payload = {
      name: form.name,
      title: form.title,
      parent_id: form.parent_id,
      icon: form.icon || null,
      path: form.path || null,
      component: form.component || null,
      type: form.type,
      permission: form.permission || null,
      sort: form.sort,
    }
    if (form.id) await updateMenu(form.id, payload)
    else await createMenu(payload)
    ElMessage.success('ok')
    dialogVisible.value = false
    load()
  } catch (e: any) {
    error(e?.message || "common.fail")
  } finally {
    submitting.value = false
  }
}

async function del(row: Menu) {
  const ok = await confirm({ message: "sys.rbac.confirmDeleteChildren" })
  if (!ok) return
  try {
    await deleteMenu(row.id)
    success("common.success")
    load()
  } catch (e: any) {
    error(e?.message || "common.fail")
  }
}

async function batchDelete() {
  if (!selectedIds.value.length) return
  const ok = await confirm({
    message: "sys.rbac.batchDeleteConfirm",
    params: { n: selectedIds.value.length },
  })
  if (!ok) return
  try {
    const msg = await batchDeleteMenus(selectedIds.value)
    success(typeof msg === "string" ? msg : "common.success")
    selectedIds.value = []
    tableRef.value?.clearSelection()
    load()
  } catch (e: any) {
    error(e?.message || "common.fail")
  }
}
</script>

<style scoped>
.search-bar {
  display: flex;
  gap: 12px;
  align-items: center;
  margin-bottom: 12px;
}
.toolbar {
  display: flex;
  gap: 12px;
  align-items: center;
  margin-bottom: 12px;
}
.hint {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}
</style>
