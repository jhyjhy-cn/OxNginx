<template>
  <div class="rbac-page">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>{{ $t('menu.rbacRoles') }}</span>
          <el-button type="primary" @click="showCreate = true">{{ $t('common.add') }}</el-button>
        </div>
      </template>

      <el-table :data="roles" v-loading="loading">
        <el-table-column prop="id" label="ID" width="60" />
        <el-table-column prop="code" label="编码" width="160" />
        <el-table-column prop="name" label="名称" />
        <el-table-column prop="remark" label="备注" />
        <el-table-column :label="$t('common.action')" width="160">
          <template #default="{ row }">
            <el-button size="small" :disabled="row.code === 'super_admin'"
              @click="$router.push(`/settings/rbac/role/${row.id}`)">菜单权限</el-button>
            <el-button size="small" type="danger" :disabled="row.code === 'super_admin'"
              @click="del(row)">{{ $t('common.delete') }}</el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <el-dialog v-model="showCreate" :title="$t('rbac.createRole')" width="400px">
      <el-form :model="form" label-width="80px">
        <el-form-item label="编码"><el-input v-model="form.code" /></el-form-item>
        <el-form-item label="名称"><el-input v-model="form.name" /></el-form-item>
        <el-form-item label="备注"><el-input v-model="form.remark" /></el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showCreate = false">{{ $t('common.cancel') }}</el-button>
        <el-button type="primary" @click="submit">{{ $t('common.confirm') }}</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import api from '@/api'

const roles = ref<any[]>([])
const loading = ref(false)
const showCreate = ref(false)
const form = reactive({ code: '', name: '', remark: '' })

onMounted(load)

async function load() {
  loading.value = true
  try {
    const { data } = await api.get('/api/rbac/roles')
    if (data.code === 0) roles.value = data.data
  } finally {
    loading.value = false
  }
}

async function submit() {
  if (!form.code || !form.name) return
  const { data } = await api.post('/api/rbac/roles', form)
  if (data.code === 0) {
    ElMessage.success('ok')
    showCreate.value = false
    form.code = ''; form.name = ''; form.remark = ''
    load()
  } else ElMessage.error(data.message)
}

async function del(row: any) {
  try {
    await ElMessageBox.confirm(`确定删除 ${row.name}?`, '提示', { type: 'warning' })
    const { data } = await api.delete(`/api/rbac/roles/${row.id}`)
    if (data.code === 0) { ElMessage.success('ok'); load() }
    else ElMessage.error(data.message)
  } catch {}
}
</script>

<style scoped>
.card-header { display: flex; justify-content: space-between; align-items: center; }
</style>