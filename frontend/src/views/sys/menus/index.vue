<template>
  <div class="menus-page h-full">
    <el-card class="h-full">
      <!-- 搜索栏 -->
      <div class="search-bar">
        <OnFormGrid :model="searchForm" :fields="searchFields" style="flex: 1" />
        <el-button type="primary" @click="search">{{ $t('common.search') }}</el-button>
        <el-button @click="reset">{{ $t('common.reset') }}</el-button>
      </div>

      <OnTable
        ref="tableRef"
        :data="treeData"
        :columns="tableColumns"
        :loading="loading"
        :pagination="false"
        :options="{ height: 'auto', rowKey: 'id' }"
        default-expand-all
        :tree-props="{ children: 'children' }"
        @selectionChange="onSelect"
        @command="handleCommand"
        @reload="load"
      >
        <template #toolbar-left>
          <el-button type="primary" @click="openCreate(null)">{{ $t('common.add') }}</el-button>
          <el-button type="danger" :disabled="!selectedIds.length" @click="batchDelete">
            {{ $t('sys.rbac.batchDelete') }} ({{ selectedIds.length }})
          </el-button>
          <span class="hint" style="margin-left: 8px">{{ $t('sys.rbac.hintMenuRefresh') }}</span>
        </template>
        <template #name="{ row }">
          <el-icon v-if="row.icon" style="margin-right: 4px; vertical-align: middle"><component :is="row.icon" /></el-icon>
          <span>{{ row.name }}</span>
        </template>
        <template #title="{ row }">{{ $t(row.title) }}</template>
        <template #type="{ row }">
          <el-tag size="small" :type="typeColor(row.type)">
            {{ $t(row.type === MenuType.Directory ? 'sys.rbac.typeM' : row.type === MenuType.Menu ? 'sys.rbac.typeC' : 'sys.rbac.typeF') }}
          </el-tag>
        </template>
      </OnTable>
    </el-card>

    <OnDialog v-model="showForm" :title="form.id ? $t('common.edit') : $t('common.add')" width="600px">
      <OnForm ref="formRef" :model="form">
        <el-form-item :label="$t('sys.rbac.colType')" prop="type" label-width="120px">
          <el-radio-group v-model="form.type">
            <el-radio-button :value="MenuType.Directory">{{ $t('sys.rbac.typeM') }}</el-radio-button>
            <el-radio-button :value="MenuType.Menu">{{ $t('sys.rbac.typeC') }}</el-radio-button>
            <el-radio-button :value="MenuType.Button">{{ $t('sys.rbac.typeF') }}</el-radio-button>
          </el-radio-group>
        </el-form-item>
        <!-- ponytail: 父级用 el-tree-select，OnFormItem 无此类型，走默认插槽 -->
        <el-form-item :label="$t('sys.rbac.colParent')" prop="parent_id" label-width="120px">
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
        <OnFormGrid :fields="formFields" :model="form" :label-width="120" />
      </OnForm>
      <template #footer>
        <el-button @click="showForm = false">{{ $t('common.cancel') }}</el-button>
        <el-button type="primary" @click="submit">{{ $t('common.confirm') }}</el-button>
      </template>
    </OnDialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import OnForm from '@/components/OnForm/OnForm/index.vue'
import OnFormGrid from '@/components/OnForm/OnFormGrid/index.vue'
import OnDialog from '@/components/OnDialog/index.vue'
import OnTable from '@/components/OnTable/index.vue'
import type { FormField } from '@/components/OnForm/types'
import type { TableColumn } from '@/components/OnTable/types'
import { listMenus, createMenu, updateMenu, deleteMenu, batchDeleteMenus } from '@/api/sys/menus'
import type { MenuItem } from '@/api/sys/menus/type'
import { useCrud, useMessage } from '@/hooks'
import { MenuType } from '@/consts'

const { success, error, confirm } = useMessage()

type Menu = MenuItem

const tableRef = ref<InstanceType<typeof OnTable>>()
const selectedIds = ref<number[]>([])
const showForm = ref(false)
const formRef = ref<InstanceType<typeof OnForm>>()

const form = reactive({
  id: null as number | null,
  type: MenuType.Menu as MenuType,
  parent_id: null as number | null,
  name: '',
  title: '',
  path: '',
  component: '',
  permission: '',
  icon: '',
  sort: 0,
})

// ponytail: 菜单接口返回整树扁平数组，无分页
const { loading, dataList, searchForm, load, search, reset } = useCrud({
  getListApi: listMenus,
  isPage: false,
})

const searchFields: FormField[] = [
  { prop: 'keyword', label: 'common.search', type: 'input', span: 8 },
]

// flat → tree；搜索时展示扁平结果
const treeData = computed<Menu[]>(() => {
  const list = dataList.value as Menu[]
  if (searchForm.keyword) return list
  const map = new Map<number, Menu>()
  list.forEach((m) => map.set(m.id, { ...m, children: [] }))
  const roots: Menu[] = []
  for (const m of map.values()) {
    if (m.parent_id && map.has(m.parent_id)) map.get(m.parent_id)!.children!.push(m)
    else roots.push(m)
  }
  return roots
})

const parentOptions = computed(() => {
  const filter = (nodes: Menu[]): Menu[] =>
    nodes.filter((n) => n.type !== MenuType.Button).map((n) => ({ ...n, children: n.children ? filter(n.children) : [] }))
  return filter(treeData.value)
})

function typeColor(t: number) {
  return t === MenuType.Directory ? 'success' : t === MenuType.Menu ? undefined : 'info'
}

const isFolder = (row: Menu) => row.type === MenuType.Button

const tableColumns: TableColumn[] = [
  { type: 'selection', width: 48 },
  { prop: 'name', label: 'sys.rbac.colName', minWidth: 160, slot: 'name' },
  { prop: 'title', label: 'sys.rbac.colTitle', minWidth: 160, slot: 'title' },
  { prop: 'path', label: 'sys.rbac.colPath', minWidth: 180 },
  { prop: 'component', label: 'sys.rbac.colComponent', minWidth: 140 },
  { prop: 'type', label: 'sys.rbac.colType', width: 80, slot: 'type' },
  { prop: 'permission', label: 'sys.rbac.colPermission', minWidth: 180 },
  { prop: 'sort', label: 'sys.rbac.colSort', width: 80 },
  {
    label: 'common.action',
    width: 200,
    fixed: 'right',
    buttons: [
      { name: 'sys.rbac.subItem', command: 'sub', size: 'small', disabled: isFolder },
      { name: 'common.edit', command: 'edit', size: 'small' },
      { name: 'common.delete', command: 'delete', type: 'danger', size: 'small' },
    ],
  },
]

const formFields = computed<FormField[]>(() => [
  { prop: 'name', label: 'sys.rbac.colName', type: 'input', required: true },
  { prop: 'title', label: 'sys.rbac.colTitle', type: 'input', required: true, placeholder: 'sys.menu.sites' },
  { prop: 'path', label: 'sys.rbac.colPath', type: 'input', visible: form.type !== MenuType.Button, placeholder: '/sites' },
  { prop: 'component', label: 'sys.rbac.colComponent', type: 'input', visible: form.type === MenuType.Menu, placeholder: 'sites/index' },
  { prop: 'permission', label: 'sys.rbac.colPermission', type: 'input', placeholder: 'sys:site:view' },
  { prop: 'icon', label: 'sys.rbac.colIcon', type: 'input', visible: form.type !== MenuType.Button, placeholder: 'House / Connection / Lock...' },
  { prop: 'sort', label: 'sys.rbac.colSort', type: 'number', min: 0, max: 9999 },
])

function handleCommand(command: string | number, row: Menu) {
  if (command === 'sub') openCreate(row)
  else if (command === 'edit') openEdit(row)
  else if (command === 'delete') del(row)
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
  Object.assign(form, {
    id: null,
    type: MenuType.Menu,
    parent_id: parent?.id ?? null,
    name: '',
    title: '',
    path: '',
    component: '',
    permission: '',
    icon: '',
    sort: 0,
  })
  showForm.value = true
}

function openEdit(row: Menu) {
  Object.assign(form, {
    id: row.id,
    type: row.type,
    parent_id: row.parent_id,
    name: row.name,
    title: row.title,
    path: row.path ?? '',
    component: row.component ?? '',
    permission: row.permission ?? '',
    icon: row.icon ?? '',
    sort: row.sort,
  })
  showForm.value = true
}

async function submit() {
  if (!formRef.value) return
  try {
    await formRef.value.validate()
  } catch {
    return
  }
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
  try {
    if (form.id) await updateMenu(form.id, payload)
    else await createMenu(payload)
    success('common.success')
    showForm.value = false
    load()
  } catch (e: any) {
    error(e?.message || 'common.fail')
  }
}

async function del(row: Menu) {
  const ok = await confirm({ message: 'sys.rbac.confirmDeleteChildren' })
  if (!ok) return
  try {
    await deleteMenu(row.id)
    success('common.success')
    load()
  } catch (e: any) {
    error(e?.message || 'common.fail')
  }
}

async function batchDelete() {
  if (!selectedIds.value.length) return
  const ok = await confirm({
    message: 'sys.rbac.batchDeleteConfirm',
    params: { n: selectedIds.value.length },
  })
  if (!ok) return
  try {
    const msg = await batchDeleteMenus(selectedIds.value)
    success(typeof msg === 'string' ? msg : 'common.success')
    selectedIds.value = []
    tableRef.value?.tableRef?.clearSelection()
    load()
  } catch (e: any) {
    error(e?.message || 'common.fail')
  }
}

onMounted(() => {
  load()
})
</script>

<style scoped>
.search-bar {
  display: flex;
  gap: 12px;
  align-items: flex-start;
  margin-bottom: 12px;
}
.hint {
  font-size: 12px;
  color: var(--el-text-color-secondary);
}
</style>
