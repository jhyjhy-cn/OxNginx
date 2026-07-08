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

      <el-table :data="roles" v-loading="loading" max-height="calc(100vh - 340px)">
        <el-table-column prop="id" label="ID" width="60" />
        <el-table-column prop="code" :label="$t('rbac.colCode')" width="160" />
        <el-table-column prop="name" :label="$t('rbac.colName')" />
        <el-table-column prop="remark" :label="$t('rbac.colRemark')" />
        <el-table-column :label="$t('common.action')" width="200">
          <template #default="{ row }">
            <el-button size="small" :disabled="row.code === 'super_admin'"
              @click="$router.push(`/settings/rbac/role/${row.id}`)">{{ $t('rbac.menuPermission') }}</el-button>
            <el-button size="small" type="danger" :disabled="row.code === 'super_admin'"
              @click="del(row)">{{ $t('common.delete') }}</el-button>
          </template>
        </el-table-column>
      </el-table>

      <OnPagination v-model:current-page="currentPage" v-model:page-size="pageSize" :total="total" @change="load" />
    </el-card>

    <el-dialog v-model="showCreate" :title="$t('rbac.createRole')" width="400px">
      <el-form :model="form" label-width="80px">
        <el-form-item :label="$t('rbac.colCode')"><el-input v-model="form.code" /></el-form-item>
        <el-form-item :label="$t('rbac.colName')"><el-input v-model="form.name" /></el-form-item>
        <el-form-item :label="$t('rbac.colRemark')"><el-input v-model="form.remark" /></el-form-item>
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
const roles = ref<any[]>([])
const loading = ref(false)
const showCreate = ref(false)
const form = reactive({ code: '', name: '', remark: '' })
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
    const { data } = await api.get('/api/rbac/roles', { params })
    if (data.code === 0) {
      roles.value = data.data.list
      total.value = data.data.total
    }
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
    await ElMessageBox.confirm(t('common.confirmDelete'), t('common.tip'), { type: 'warning' })
    const { data } = await api.delete(`/api/rbac/roles/${row.id}`)
    if (data.code === 0) { ElMessage.success('ok'); load() }
    else ElMessage.error(data.message)
  } catch {}
}
</script>

<style scoped>
.search-bar { display: flex; gap: 12px; align-items: center; margin-bottom: 12px; }
.toolbar { display: flex; gap: 12px; align-items: center; margin-bottom: 12px; }
</style>
