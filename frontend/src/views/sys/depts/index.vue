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
        <el-button type="primary" @click="openCreate(null)">{{ $t('common.add') }}</el-button>
        <el-button @click="load">{{ $t('common.refresh') }}</el-button>
      </div>

      <el-table :data="depts" v-loading="loading" row-key="id" :tree-props="{ children: 'children' }" max-height="calc(100vh - 380px)">
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
            <el-button type="primary" text size="small" @click="openCreate(row)">+{{ $t('rbac.subItem') }}</el-button>
            <el-button type="primary" text size="small" @click="openEdit(row)">{{ $t('common.edit') }}</el-button>
            <el-button type="danger" text size="small" @click="del(row)">{{ $t('common.delete') }}</el-button>
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
import { useI18n } from 'vue-i18n'
import api from '@/api'
import OnPagination from '@/components/OnPagination/index.vue'
import OnDialog from '@/components/OnDialog/index.vue'

const { t } = useI18n()

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
const rules = { name: [{ required: true, message: t('rbac.required'), trigger: 'blur' }] }
const keyword = ref('')
const currentPage = ref(1)
const pageSize = ref(100)
const total = ref(0)

const parentOptions = computed(() => {
  const filter = (nodes: Dept[]): Dept[] =>
    nodes.filter((n) => n.id !== form.id).map((n) => ({ ...n, children: n.children ? filter(n.children) : [] }))
  return filter(allDepts.value)
})

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
    const params: Record<string, string | number> = { page: currentPage.value, page_size: pageSize.value }
    if (keyword.value) params.keyword = keyword.value
    const { data } = await api.get('/api/rbac/depts', { params })
    if (data.code !== 0) return
    const list: Dept[] = data.data.list || data.data
    total.value = data.data.total || list.length
    const map = new Map<number, Dept>()
    list.forEach((m) => map.set(m.id, { ...m, children: [] }))
    const roots: Dept[] = []
    for (const m of map.values()) {
      if (m.parent_id && map.has(m.parent_id)) map.get(m.parent_id)!.children!.push(m)
      else roots.push(m)
    }
    allDepts.value = list
    depts.value = keyword.value ? list : roots
  } finally {
    loading.value = false
  }
}

function openCreate(parent: Dept | null) {
  form.id = null
  form.parent_id = parent?.id ?? null
  form.name = ''
  form.sort = 0
  dialogVisible.value = true
  nextTick(() => formRef.value?.clearValidate())
}

function openEdit(row: Dept) {
  form.id = row.id
  form.parent_id = row.parent_id
  form.name = row.name
  form.sort = row.sort
  dialogVisible.value = true
  nextTick(() => formRef.value?.clearValidate())
}

async function submit() {
  const valid = await formRef.value?.validate().catch(() => false)
  if (!valid) return
  submitting.value = true
  try {
    const payload = { name: form.name, parent_id: form.parent_id, sort: form.sort }
    const { data } = form.id ? await api.put(`/api/rbac/depts/${form.id}`, payload) : await api.post('/api/rbac/depts', payload)
    if (data.code === 0) {
      ElMessage.success('ok')
      dialogVisible.value = false
      load()
    } else ElMessage.error(data.message)
  } finally {
    submitting.value = false
  }
}

async function del(row: Dept) {
  try {
    await ElMessageBox.confirm(t('common.confirmDelete'), t('common.tip'), { type: 'warning' })
    const { data } = await api.delete(`/api/rbac/depts/${row.id}`)
    if (data.code === 0) {
      ElMessage.success('ok')
      load()
    } else ElMessage.error(data.message)
  } catch {}
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
</style>
