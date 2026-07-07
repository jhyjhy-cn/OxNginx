<template>
  <div class="rbac-page">
    <el-card>
      <template #header><span>{{ $t('menu.rbacUsers') }}</span></template>
      <el-button type="primary" @click="showCreate = true">{{ $t('common.add') }}</el-button>

      <el-table :data="users" v-loading="loading" style="margin-top: 16px">
        <el-table-column prop="id" label="ID" width="60" />
        <el-table-column prop="username" :label="$t('login.username')" />
        <el-table-column prop="roles" label="角色" />
        <el-table-column prop="disabled" label="状态" width="80">
          <template #default="{ row }">
            <el-tag :type="row.disabled ? 'danger' : 'success'" size="small">
              {{ row.disabled ? $t('common.disabled') : $t('common.enabled') }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column :label="$t('common.action')" width="200">
          <template #default="{ row }">
            <el-button size="small" @click="resetPwd(row)">{{ $t('rbac.resetPassword') }}</el-button>
            <el-button size="small" type="danger" :disabled="row.username === 'admin'"
              @click="del(row)">{{ $t('common.delete') }}</el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <el-dialog v-model="showCreate" :title="$t('rbac.createUser')" width="400px">
      <el-form :model="form" label-width="80px">
        <el-form-item :label="$t('login.username')">
          <el-input v-model="form.username" />
        </el-form-item>
        <el-form-item :label="$t('login.password')">
          <el-input v-model="form.password" type="password" show-password />
        </el-form-item>
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

const users = ref<any[]>([])
const loading = ref(false)
const showCreate = ref(false)
const form = reactive({ username: '', password: '' })

onMounted(load)

async function load() {
  loading.value = true
  try {
    const { data } = await api.get('/api/rbac/users')
    if (data.code === 0) users.value = data.data
  } finally {
    loading.value = false
  }
}

async function submit() {
  if (!form.username || !form.password) return
  const { data } = await api.post('/api/rbac/users', form)
  if (data.code === 0) {
    ElMessage.success('ok')
    showCreate.value = false
    form.username = ''
    form.password = ''
    load()
  } else {
    ElMessage.error(data.message)
  }
}

async function resetPwd(row: any) {
  try {
    const { value } = await ElMessageBox.prompt('新密码', '重置密码', { inputValue: '123456' })
    const { data } = await api.post(`/api/rbac/users/${row.id}/reset-password`, { new_password: value })
    if (data.code === 0) ElMessage.success('ok')
    else ElMessage.error(data.message)
  } catch {}
}

async function del(row: any) {
  try {
    await ElMessageBox.confirm(`确定删除 ${row.username}?`, '提示', { type: 'warning' })
    const { data } = await api.delete(`/api/rbac/users/${row.id}`)
    if (data.code === 0) { ElMessage.success('ok'); load() }
    else ElMessage.error(data.message)
  } catch {}
}
</script>

<style scoped>
.rbac-page { padding: 0; }
</style>