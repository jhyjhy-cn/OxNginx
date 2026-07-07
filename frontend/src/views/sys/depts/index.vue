<template>
  <div class="rbac-page">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>{{ $t('menu.rbacDepts') }}</span>
        </div>
      </template>

      <div class="toolbar">
        <el-button type="primary" @click="openCreate(null)">{{ $t('common.add') }}</el-button>
        <el-button @click="load">{{ $t('common.refresh') }}</el-button>
      </div>

      <el-table :data="depts" v-loading="loading" row-key="id" :tree-props="{ children: 'children' }">
        <el-table-column prop="name" :label="$t('rbac.colName')" min-width="200" />
        <el-table-column prop="sort" :label="$t('rbac.colSort')" width="100" />
        <el-table-column prop="status" :label="$t('common.status')" width="100">
          <template #default="{ row }">
            <el-tag size="small" :type="row.status === 'enabled' ? 'success' : 'info'">
              {{ row.status === 'enabled' ? $t('common.enabled') : $t('common.disabled') }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column :label="$t('common.action')" width="200" fixed="right">
          <template #default="{ row }">
            <el-button type="primary" text size="small" @click="openCreate(row)">+子项</el-button>
            <el-button type="primary" text size="small" @click="openEdit(row)">{{ $t('common.edit') }}</el-button>
            <el-button type="danger" text size="small" @click="del(row)">{{ $t('common.delete') }}</el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <OnDialog v-model="dialogVisible" :title="form.id ? $t('common.edit') : $t('common.add')" width="450px">
      <el-form :model="form" label-width="80px" :rules="rules" ref="formRef">
        <el-form-item :label="$t('rbac.colParent')">
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
import api from '@/api'
import OnDialog from '@/components/OnDialog/index.vue'

interface Dept {
  id: number
  parent_id: number | null
  name: string
  sort: number
  status: string
  children?: Dept[]
}

const allDepts = ref<Dept[]>([])
const depts = ref<Dept[]>([])
const loading = ref(false)
const dialogVisible = ref(false)
const submitting = ref(false)
const formRef = ref()
const form = reactive({ id: null as number | null, parent_id: null as number | null, name: '', sort: 0 })
const rules = { name: [{ required: true, message: '必填', trigger: 'blur' }] }

const parentOptions = computed(() => {
  const filter = (nodes: Dept[]): Dept[] =>
    nodes.filter(n => n.id !== form.id).map(n => ({ ...n, children: n.children ? filter(n.children) : [] }))
  return filter(allDepts.value)
})

onMounted(load)

async function load() {
  loading.value = true
  try {
    const { data } = await api.get('/api/rbac/depts')
    if (data.code !== 0) return
    const list: Dept[] = data.data
    const map = new Map<number, Dept>()
    list.forEach(m => map.set(m.id, { ...m, children: [] }))
    const roots: Dept[] = []
    for (const m of map.values()) {
      if (m.parent_id && map.has(m.parent_id)) map.get(m.parent_id)!.children!.push(m)
      else roots.push(m)
    }
    allDepts.value = list
    depts.value = roots
  } finally {
    loading.value = false
  }
}

function openCreate(parent: Dept | null) {
  form.id = null; form.parent_id = parent?.id ?? null; form.name = ''; form.sort = 0
  dialogVisible.value = true
  nextTick(() => formRef.value?.clearValidate())
}

function openEdit(row: Dept) {
  form.id = row.id; form.parent_id = row.parent_id; form.name = row.name; form.sort = row.sort
  dialogVisible.value = true
  nextTick(() => formRef.value?.clearValidate())
}

async function submit() {
  const valid = await formRef.value?.validate().catch(() => false)
  if (!valid) return
  submitting.value = true
  try {
    const payload = { name: form.name, parent_id: form.parent_id, sort: form.sort }
    const { data } = form.id
      ? await api.put(`/api/rbac/depts/${form.id}`, payload)
      : await api.post('/api/rbac/depts', payload)
    if (data.code === 0) { ElMessage.success('ok'); dialogVisible.value = false; load() }
    else ElMessage.error(data.message)
  } finally {
    submitting.value = false
  }
}

async function del(row: Dept) {
  try {
    await ElMessageBox.confirm(`确定删除「${row.name}」?`, '提示', { type: 'warning' })
    const { data } = await api.delete(`/api/rbac/depts/${row.id}`)
    if (data.code === 0) { ElMessage.success('ok'); load() }
    else ElMessage.error(data.message)
  } catch {}
}
</script>

<style scoped>
.toolbar { display: flex; gap: 12px; margin-bottom: 12px; }
.card-header { display: flex; justify-content: space-between; align-items: center; }
</style>