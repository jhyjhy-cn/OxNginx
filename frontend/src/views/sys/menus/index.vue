<template>
  <div class="rbac-page">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>{{ $t('menu.rbacMenus') }}</span>
          <span class="hint">新增/修改菜单后,刷新页面或重启服务生效</span>
        </div>
      </template>

      <div class="toolbar">
        <el-button type="primary" @click="openCreate(null)">
          <el-icon><Plus /></el-icon>{{ $t('common.add') }}
        </el-button>
        <el-button
          type="danger"
          :disabled="!selectedIds.length"
          @click="batchDelete"
        >
          <el-icon><Delete /></el-icon>批量删除 ({{ selectedIds.length }})
        </el-button>
        <el-button @click="load">{{ $t('common.refresh') }}</el-button>
      </div>

      <el-table
        :data="menus"
        v-loading="loading"
        row-key="id"
        :tree-props="{ children: 'children' }"
        @selection-change="onSelect"
        ref="tableRef"
      >
        <el-table-column type="selection" width="48" />
        <el-table-column prop="name" :label="$t('rbac.colName')" min-width="160">
          <template #default="{ row }">
            <el-icon v-if="row.icon" style="margin-right: 4px; vertical-align: middle;"><component :is="row.icon" /></el-icon>
            <span>{{ row.name }}</span>
          </template>
        </el-table-column>
        <el-table-column prop="title" :label="$t('rbac.colTitle')" min-width="160">
          <template #default="{ row }">
            <span>{{ $t(row.title) }}</span>
          </template>
        </el-table-column>
        <el-table-column prop="path" :label="$t('rbac.colPath')" min-width="180" />
        <el-table-column prop="component" :label="$t('rbac.colComponent')" min-width="140" />
        <el-table-column prop="type" :label="$t('rbac.colType')" width="80">
          <template #default="{ row }">
            <el-tag size="small" :type="typeColor(row.type)">{{ typeLabel(row.type) }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="permission" :label="$t('rbac.colPermission')" min-width="180" />
        <el-table-column prop="sort" :label="$t('rbac.colSort')" width="80" />
        <el-table-column :label="$t('common.action')" width="200" fixed="right">
          <template #default="{ row }">
            <el-button v-if="row.type !== 'F'" type="primary" text size="small" @click="openCreate(row)">
              +子项
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
    </el-card>

    <OnDialog v-model="dialogVisible" :title="form.id ? $t('common.edit') : $t('common.add')" width="600px">
      <el-form :model="form" label-width="120px" :rules="rules" ref="formRef">
        <el-form-item :label="$t('rbac.colType')" prop="type">
          <el-radio-group v-model="form.type">
            <el-radio-button value="M">{{ $t('rbac.typeM') }}</el-radio-button>
            <el-radio-button value="C">{{ $t('rbac.typeC') }}</el-radio-button>
            <el-radio-button value="F">{{ $t('rbac.typeF') }}</el-radio-button>
          </el-radio-group>
        </el-form-item>
        <el-form-item :label="$t('rbac.colParent')" prop="parent_id">
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
        <el-form-item :label="$t('rbac.colName')" prop="name">
          <el-input v-model="form.name" />
        </el-form-item>
        <el-form-item :label="$t('rbac.colTitle') + ' (i18n)'" prop="title">
          <el-input v-model="form.title" placeholder="menu.sites" />
        </el-form-item>
        <el-form-item :label="$t('rbac.colPath')" v-if="form.type !== 'F'">
          <el-input v-model="form.path" placeholder="/sites" />
        </el-form-item>
        <el-form-item :label="$t('rbac.colComponent')" v-if="form.type === 'C'">
          <el-input v-model="form.component" placeholder="sites/index" />
        </el-form-item>
        <el-form-item :label="$t('rbac.colPermission')">
          <el-input v-model="form.permission" placeholder="sys:site:view" />
        </el-form-item>
        <el-form-item :label="$t('rbac.colIcon')" v-if="form.type !== 'F'">
          <el-input v-model="form.icon" placeholder="House / Connection / Lock..." />
        </el-form-item>
        <el-form-item :label="$t('rbac.colSort')">
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
import { ElMessage, ElMessageBox } from 'element-plus'
import { Plus, Delete } from '@element-plus/icons-vue'
import api from '@/api'
import OnDialog from '@/components/OnDialog/index.vue'

interface Menu {
  id: number
  parent_id: number | null
  name: string
  title: string
  icon: string | null
  path: string | null
  component: string | null
  type: 'M' | 'C' | 'F'
  permission: string | null
  sort: number
  children?: Menu[]
}

const allMenus = ref<Menu[]>([])
const menus = ref<Menu[]>([])
const loading = ref(false)
const tableRef = ref()
const selectedIds = ref<number[]>([])

const parentOptions = computed(() => {
  const filter = (nodes: Menu[]): Menu[] =>
    nodes
      .filter(n => n.type !== 'F')
      .map(n => ({ ...n, children: n.children ? filter(n.children) : [] }))
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
  name: [{ required: true, message: '必填', trigger: 'blur' }],
  title: [{ required: true, message: '必填', trigger: 'blur' }],
  type: [{ required: true, message: '必填', trigger: 'change' }],
}

function typeColor(t: string) {
  return t === 'M' ? 'success' : t === 'C' ? '' : 'info'
}
function typeLabel(t: string) {
  return t === 'M' ? '目录' : t === 'C' ? '菜单' : '按钮'
}

onMounted(load)

async function load() {
  loading.value = true
  try {
    const { data } = await api.get('/api/rbac/menus')
    if (data.code !== 0) return
    const list: Menu[] = data.data
    const map = new Map<number, Menu>()
    list.forEach(m => map.set(m.id, { ...m, children: [] }))
    const roots: Menu[] = []
    for (const m of map.values()) {
      if (m.parent_id && map.has(m.parent_id)) {
        map.get(m.parent_id)!.children!.push(m)
      } else {
        roots.push(m)
      }
    }
    allMenus.value = list
    menus.value = roots
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
    const { data } = form.id
      ? await api.put(`/api/rbac/menus/${form.id}`, payload)
      : await api.post('/api/rbac/menus', payload)
    if (data.code === 0) {
      ElMessage.success('ok')
      dialogVisible.value = false
      load()
    } else {
      ElMessage.error(data.message)
    }
  } finally {
    submitting.value = false
  }
}

async function del(row: Menu) {
  try {
    await ElMessageBox.confirm(
      `确定删除「${row.title}」? 子菜单和关联角色权限将一并删除`,
      '提示',
      { type: 'warning' },
    )
    const { data } = await api.delete(`/api/rbac/menus/${row.id}`)
    if (data.code === 0) {
      ElMessage.success('ok')
      load()
    } else {
      ElMessage.error(data.message)
    }
  } catch {}
}

async function batchDelete() {
  if (!selectedIds.value.length) return
  try {
    await ElMessageBox.confirm(
      `确定批量删除 ${selectedIds.value.length} 项? 子菜单将一并删除`,
      '提示',
      { type: 'warning' },
    )
    const { data } = await api.post('/api/rbac/menus/batch-delete', selectedIds.value)
    if (data.code === 0) {
      ElMessage.success(data.data || 'ok')
      selectedIds.value = []
      tableRef.value?.clearSelection()
      load()
    } else {
      ElMessage.error(data.message)
    }
  } catch {}
}
</script>

<style scoped>
.toolbar {
  display: flex;
  gap: 12px;
  margin-bottom: 12px;
}
.card-header { display: flex; justify-content: space-between; align-items: center; }
.hint { font-size: 12px; color: var(--el-text-color-secondary); }
</style>