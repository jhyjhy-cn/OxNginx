<template>
  <div class="depts-page h-full">
    <el-card class="h-full">
      <!-- 搜索栏 -->
      <div class="search-bar">
        <OnFormGrid :model="searchForm" :fields="searchFields" style="flex: 1" />
        <el-button type="primary" @click="search">{{ $t('common.search') }}</el-button>
        <el-button @click="reset">{{ $t('common.reset') }}</el-button>
      </div>

      <OnTable
        :data="treeData"
        :columns="tableColumns"
        :loading="loading"
        :pagination="{ total, currentPage: page, pageSize }"
        :options="{ height: 'auto', rowKey: 'id' }"
        default-expand-all
        :tree-props="{ children: 'children' }"
        @page-change="onPageChange"
        @command="handleCommand"
        @reload="load"
        @selectionChange="(rows: any[]) => (selectedRows = rows)"
      >
        <template #toolbar-left>
          <el-button type="primary" @click="openCreate(null)">{{ $t('common.add') }}</el-button>
          <el-button
            type="danger"
            :disabled="!selectedRows.length"
            @click="batchDelete"
          >
            {{ $t('common.batchDelete') }} ({{ selectedRows.length }})
          </el-button>
        </template>
        <template #status="{ row }">
          <el-tag size="small" :type="row.status === 'enabled' ? 'success' : 'info'">
            {{ row.status === 'enabled' ? $t('common.enabled') : $t('common.disabled') }}
          </el-tag>
        </template>
      </OnTable>
    </el-card>

    <OnDialog v-model="showForm" :title="form.id ? $t('common.edit') : $t('common.add')" width="450px">
      <OnForm ref="formRef" :model="form">
        <!-- ponytail: 父级用 el-tree-select，OnFormItem 无此类型，走默认插槽 -->
        <el-form-item :label="$t('sys.rbac.colParent')" prop="parent_id" label-width="100px">
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
        <OnFormGrid :fields="formFields" :model="form" />
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
import { listDepts, createDept, updateDept, deleteDept, batchDeleteDepts } from '@/api/sys/depts'
import type { Dept } from '@/api/sys/depts/type'
import { useCrud, useMessage } from '@/hooks'

const { success, error, confirm } = useMessage()

const showForm = ref(false)
const selectedRows = ref<any[]>([])
const formRef = ref<InstanceType<typeof OnForm>>()
const form = reactive({ id: null as number | null, parent_id: null as number | null, name: '', sort: 0 })

const {
  loading,
  dataList,
  total,
  page,
  pageSize,
  searchForm,
  load,
  search,
  reset,
} = useCrud({
  getListApi: listDepts,
  isPage: true,
  pageSize: 100,
})

const searchFields: FormField[] = [
  { prop: 'keyword', label: 'common.search', type: 'input', span: 8 },
]

// flat → tree；搜索时展示扁平结果
const treeData = computed<Dept[]>(() => {
  const list = dataList.value as Dept[]
  if (searchForm.keyword) return list
  const map = new Map<number, Dept>()
  list.forEach((m) => map.set(m.id, { ...m, children: [] }))
  const roots: Dept[] = []
  for (const m of map.values()) {
    if (m.parent_id && map.has(m.parent_id)) map.get(m.parent_id)!.children!.push(m)
    else roots.push(m)
  }
  return roots
})

const parentOptions = computed(() => {
  const filter = (nodes: Dept[]): Dept[] =>
    nodes.filter((n) => n.id !== form.id).map((n) => ({ ...n, children: n.children ? filter(n.children) : [] }))
  return filter(treeData.value)
})

const tableColumns: TableColumn[] = [
  { type: 'selection', width: 48 },
  { prop: 'name', label: 'sys.rbac.colName', minWidth: 200 },
  { prop: 'sort', label: 'sys.rbac.colSort', width: 100 },
  { prop: 'status', label: 'common.status', width: 100, slot: 'status' },
  {
    label: 'common.action',
    width: 200,
    fixed: 'right',
    buttons: [
      { name: 'sys.rbac.subItem', command: 'sub', size: 'small' },
      { name: 'common.edit', command: 'edit', size: 'small' },
      { name: 'common.delete', command: 'delete', type: 'danger', size: 'small' },
    ],
  },
]

const formFields: FormField[] = [
  { prop: 'name', label: 'sys.rbac.colName', type: 'input', required: true },
  { prop: 'sort', label: 'sys.rbac.colSort', type: 'number', min: 0, max: 9999 },
]

function onPageChange(p: number) {
  page.value = p
  load()
}

function handleCommand(command: string | number, row: Dept) {
  if (command === 'sub') openCreate(row)
  else if (command === 'edit') openEdit(row)
  else if (command === 'delete') del(row)
}

function openCreate(parent: Dept | null) {
  Object.assign(form, { id: null, parent_id: parent?.id ?? null, name: '', sort: 0 })
  showForm.value = true
}

function openEdit(row: Dept) {
  Object.assign(form, { id: row.id, parent_id: row.parent_id, name: row.name, sort: row.sort })
  showForm.value = true
}

async function submit() {
  if (!formRef.value) return
  try {
    await formRef.value.validate()
  } catch {
    return
  }
  const payload = { name: form.name, parent_id: form.parent_id, sort: form.sort }
  try {
    if (form.id) await updateDept(form.id, payload)
    else await createDept(payload)
    success('common.success')
    showForm.value = false
    load()
  } catch (e: any) {
    error(e?.message || 'common.fail')
  }
}

async function del(row: Dept) {
  const ok = await confirm({ message: 'common.confirmDelete' })
  if (!ok) return
  try {
    await deleteDept(row.id)
    success('common.success')
    load()
  } catch (e: any) {
    error(e?.message || 'common.fail')
  }
}

async function batchDelete() {
  if (!selectedRows.value.length) return
  const ok = await confirm({
    message: 'sys.rbac.confirmBatchDelete',
    params: { n: selectedRows.value.length },
  })
  if (!ok) return
  try {
    const ids = selectedRows.value.map((r) => r.id)
    const msg = await batchDeleteDepts(ids)
    success(typeof msg === 'string' ? msg : 'common.success')
    selectedRows.value = []
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
</style>
