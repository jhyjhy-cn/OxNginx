<template>
  <div class="rbac-page">
    <el-card>
      <div class="search-bar">
        <el-input v-model="keyword" :placeholder="$t('common.search')" clearable style="width: 240px" @input="onInput" @keyup.enter="doSearch" />
        <el-button type="primary" @click="doSearch">{{ $t('common.search') }}</el-button>
        <el-button @click="doReset">{{ $t('common.reset') }}</el-button>
      </div>

      <div class="toolbar">
        <el-button type="primary" @click="showCreate = true">{{ $t('common.add') }}</el-button>
      </div>

      <el-table :data="users" v-loading="loading" max-height="calc(100vh - 340px)">
        <el-table-column prop="id" label="ID" width="60" />
        <el-table-column prop="username" :label="$t('login.username')" />
        <el-table-column prop="roles" :label="$t('rbac.colRoles')" />
        <el-table-column prop="disabled" :label="$t('common.status')" width="80">
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

      <OnPagination v-model:current-page="currentPage" v-model:page-size="pageSize" :total="total" @change="load" />
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
import { useI18n } from 'vue-i18n'
import api from '@/api'
import OnPagination from '@/components/OnPagination/index.vue'

const { t } = useI18n()

const users = ref<any[]>([])
const loading = ref(false)
const showCreate = ref(false)
const form = reactive({ username: '', password: '' })
const keyword = ref('')
const currentPage = ref(1)
const pageSize = ref(20)
const total = ref(0)

function doSearch() { currentPage.value = 1; load() }
function doReset() { keyword.value = ''; currentPage.value = 1; load() }

let timer: ReturnType<typeof setTimeout> | null = null
function onInput() { if (timer) clearTimeout(timer); timer = setTimeout(doSearch, 300) }

onMounted(load)

async function load() {
  loading.value = true
  try {
    const params: Record<string, string | number> = { page: currentPage.value, page_size: pageSize.value }
    if (keyword.value) params.keyword = keyword.value
    const { data } = await api.get('/api/rbac/users', { params })
    if (data.code === 0) {
      users.value = data.data.list
      total.value = data.data.total
    }
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
    const { value } = await ElMessageBox.prompt(t('rbac.resetPassword'), t('rbac.resetPassword'), { inputValue: '123456' })
    const { data } = await api.post(`/api/rbac/users/${row.id}/reset-password`, { new_password: value })
    if (data.code === 0) ElMessage.success('ok')
    else ElMessage.error(data.message)
  } catch {}
}

async function del(row: any) {
  try {
    await ElMessageBox.confirm(t('common.confirmDelete'), t('common.tip'), { type: 'warning' })
    const { data } = await api.delete(`/api/rbac/users/${row.id}`)
    if (data.code === 0) { ElMessage.success('ok'); load() }
    else ElMessage.error(data.message)
  } catch {}
}
</script>

<style scoped>
.search-bar { display: flex; gap: 12px; align-items: center; margin-bottom: 12px; }
.toolbar { display: flex; gap: 12px; align-items: center; margin-bottom: 12px; }
</style>
