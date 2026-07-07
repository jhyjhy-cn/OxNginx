<template>
  <div class="rbac-page">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>{{ $t('menu.rbacPosts') }}</span>
        </div>
      </template>

      <div class="toolbar">
        <el-button type="primary" @click="openCreate">{{ $t('common.add') }}</el-button>
        <el-button @click="load">{{ $t('common.refresh') }}</el-button>
      </div>

      <el-table :data="posts" v-loading="loading">
        <el-table-column prop="code" label="code" width="160" />
        <el-table-column prop="name" :label="$t('rbac.colName')" />
        <el-table-column prop="sort" :label="$t('rbac.colSort')" width="100" />
        <el-table-column prop="status" :label="$t('common.status')" width="100">
          <template #default="{ row }">
            <el-tag size="small" :type="row.status === 'enabled' ? 'success' : 'info'">
              {{ row.status === 'enabled' ? $t('common.enabled') : $t('common.disabled') }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column :label="$t('common.action')" width="160" fixed="right">
          <template #default="{ row }">
            <el-button type="primary" text size="small" @click="openEdit(row)">{{ $t('common.edit') }}</el-button>
            <el-button type="danger" text size="small" @click="del(row)">{{ $t('common.delete') }}</el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <OnDialog v-model="dialogVisible" :title="form.id ? $t('common.edit') : $t('common.add')" width="450px">
      <el-form :model="form" label-width="80px" :rules="rules" ref="formRef">
        <el-form-item label="code" prop="code">
          <el-input v-model="form.code" :disabled="!!form.id" />
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
import { ref, reactive, onMounted, nextTick } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import api from '@/api'
import OnDialog from '@/components/OnDialog/index.vue'

interface Post { id: number; code: string; name: string; sort: number; status: string }

const posts = ref<Post[]>([])
const loading = ref(false)
const dialogVisible = ref(false)
const submitting = ref(false)
const formRef = ref()
const form = reactive({ id: null as number | null, code: '', name: '', sort: 0 })
const rules = {
  code: [{ required: true, message: '必填', trigger: 'blur' }],
  name: [{ required: true, message: '必填', trigger: 'blur' }],
}

onMounted(load)

async function load() {
  loading.value = true
  try {
    const { data } = await api.get('/api/rbac/posts')
    if (data.code === 0) posts.value = data.data
  } finally {
    loading.value = false
  }
}

function openCreate() {
  form.id = null; form.code = ''; form.name = ''; form.sort = 0
  dialogVisible.value = true
  nextTick(() => formRef.value?.clearValidate())
}

function openEdit(row: Post) {
  form.id = row.id; form.code = row.code; form.name = row.name; form.sort = row.sort
  dialogVisible.value = true
  nextTick(() => formRef.value?.clearValidate())
}

async function submit() {
  const valid = await formRef.value?.validate().catch(() => false)
  if (!valid) return
  submitting.value = true
  try {
    const payload = { code: form.code, name: form.name, sort: form.sort }
    const { data } = form.id
      ? await api.put(`/api/rbac/posts/${form.id}`, payload)
      : await api.post('/api/rbac/posts', payload)
    if (data.code === 0) { ElMessage.success('ok'); dialogVisible.value = false; load() }
    else ElMessage.error(data.message)
  } finally {
    submitting.value = false
  }
}

async function del(row: Post) {
  try {
    await ElMessageBox.confirm(`确定删除「${row.name}」?`, '提示', { type: 'warning' })
    const { data } = await api.delete(`/api/rbac/posts/${row.id}`)
    if (data.code === 0) { ElMessage.success('ok'); load() }
    else ElMessage.error(data.message)
  } catch {}
}
</script>

<style scoped>
.toolbar { display: flex; gap: 12px; margin-bottom: 12px; }
.card-header { display: flex; justify-content: space-between; align-items: center; }
</style>