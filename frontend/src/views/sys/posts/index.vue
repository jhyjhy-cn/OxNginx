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
        <el-button type="primary" @click="openCreate">{{ $t('common.add') }}</el-button>
        <el-button @click="load">{{ $t('common.refresh') }}</el-button>
      </div>

      <el-table :data="posts" v-loading="loading" max-height="calc(100vh - 340px)">
        <el-table-column prop="code" :label="$t('sys.rbac.colCode')" width="160" />
        <el-table-column prop="name" :label="$t('sys.rbac.colName')" />
        <el-table-column prop="sort" :label="$t('sys.rbac.colSort')" width="100" />
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

      <OnPagination v-model:current-page="currentPage" v-model:page-size="pageSize" :total="total" @change="load" />
    </el-card>

    <OnDialog v-model="dialogVisible" :title="form.id ? $t('common.edit') : $t('common.add')" width="450px">
      <el-form :model="form" label-width="80px" :rules="rules" ref="formRef">
        <el-form-item :label="$t('sys.rbac.colCode')" prop="code">
          <el-input v-model="form.code" :disabled="!!form.id" />
        </el-form-item>
        <el-form-item :label="$t('sys.rbac.colName')" prop="name">
          <el-input v-model="form.name" />
        </el-form-item>
        <el-form-item :label="$t('sys.rbac.colSort')">
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
import { ElMessage } from 'element-plus'
import { useI18n } from 'vue-i18n'
import OnPagination from '@/components/OnPagination/index.vue'
import OnDialog from '@/components/OnDialog/index.vue'
import {
  listPosts,
  createPost,
  updatePost,
  deletePost,
} from '@/api/sys/posts'
import type { Post } from '@/api/sys/posts/type'
import { useMessage } from '@/hooks'

const { t } = useI18n()
const { success, error, confirm } = useMessage()

const posts = ref<Post[]>([])
const loading = ref(false)
const dialogVisible = ref(false)
const submitting = ref(false)
const formRef = ref()
const form = reactive({ id: null as number | null, code: '', name: '', sort: 0 })
const rules = {
  code: [{ required: true, message: t('sys.rbac.required'), trigger: 'blur' }],
  name: [{ required: true, message: t('sys.rbac.required'), trigger: 'blur' }],
}
const keyword = ref('')
const currentPage = ref(1)
const pageSize = ref(20)
const total = ref(0)

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
    const params: Record<string, unknown> = { page: currentPage.value, page_size: pageSize.value }
    if (keyword.value) params.keyword = keyword.value
    const data = await listPosts(params)
    posts.value = data.list || []
    total.value = data.total || 0
  } catch (e: any) {
    error(e?.message || "common.fail")
  } finally {
    loading.value = false
  }
}

function openCreate() {
  form.id = null
  form.code = ''
  form.name = ''
  form.sort = 0
  dialogVisible.value = true
  nextTick(() => formRef.value?.clearValidate())
}

function openEdit(row: Post) {
  form.id = row.id
  form.code = row.code
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
    const payload = { code: form.code, name: form.name, sort: form.sort }
    if (form.id) await updatePost(form.id, payload)
    else await createPost(payload)
    ElMessage.success('ok')
    dialogVisible.value = false
    load()
  } catch (e: any) {
    error(e?.message || "common.fail")
  } finally {
    submitting.value = false
  }
}

async function del(row: Post) {
  const ok = await confirm({ message: "common.confirmDelete" })
  if (!ok) return
  try {
    await deletePost(row.id)
    success("common.success")
    load()
  } catch (e: any) {
    error(e?.message || "common.fail")
  }
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
