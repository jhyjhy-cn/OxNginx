<template>
  <div class="rbac-page">
    <el-card>
      <div class="toolbar">
        <el-button type="primary" @click="openAdd">{{ $t('common.add') }}</el-button>
        <el-button @click="load">{{ $t('common.refresh') }}</el-button>
      </div>

      <el-table :data="dicts" v-loading="loading" max-height="calc(100vh - 260px)" style="margin-top: 12px">
        <el-table-column prop="name" :label="$t('sys.dict.colName')" min-width="160" />
        <el-table-column prop="code" :label="$t('sys.dict.colCode')" min-width="160" />
        <el-table-column prop="description" :label="$t('sys.dict.colDesc')" min-width="200" />
        <el-table-column prop="status" :label="$t('common.status')" width="100">
          <template #default="{ row }">
            <el-tag :type="row.status === 'enabled' ? 'success' : 'danger'" size="small">
              {{ row.status === 'enabled' ? $t('common.enabled') : $t('common.disabled') }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column :label="$t('common.action')" width="220" fixed="right">
          <template #default="{ row }">
            <el-button type="primary" text size="small" @click="openEdit(row)">{{ $t('common.edit') }}</el-button>
            <el-button type="primary" text size="small" @click="openItems(row)">{{ $t('sys.dict.manageItems') }}</el-button>
            <el-button type="danger" text size="small" @click="del(row)">{{ $t('common.delete') }}</el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- 新增/编辑字典 -->
    <OnDialog v-model="showForm" :title="form.id ? $t('common.edit') : $t('common.add')" width="480px">
      <el-form :model="form" label-width="100px">
        <el-form-item :label="$t('sys.dict.colName')" required>
          <el-input v-model="form.name" />
        </el-form-item>
        <el-form-item :label="$t('sys.dict.colCode')" required>
          <el-input v-model="form.code" :disabled="!!form.id" />
        </el-form-item>
        <el-form-item :label="$t('sys.dict.colDesc')">
          <el-input v-model="form.description" type="textarea" :rows="2" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showForm = false">{{ $t('common.cancel') }}</el-button>
        <el-button type="primary" @click="save" :loading="saving">{{ $t('common.confirm') }}</el-button>
      </template>
    </OnDialog>

    <!-- 字典项管理 -->
    <OnDialog v-model="showItems" :title="$t('sys.dict.manageItems') + ' - ' + currentDict?.name" width="800px">
      <div v-if="currentDict" class="dict-items">
        <div class="toolbar">
          <el-button type="primary" size="small" @click="openAddItem">{{ $t('common.add') }}</el-button>
        </div>
        <el-table :data="items" v-loading="loadingItems" max-height="400">
          <el-table-column prop="label" :label="$t('sys.dict.colLabel')" min-width="160" />
          <el-table-column prop="value" :label="$t('sys.dict.colValue')" min-width="160" />
          <el-table-column prop="sort" :label="$t('sys.dict.colSort')" width="80" />
          <el-table-column prop="status" :label="$t('common.status')" width="100">
            <template #default="{ row }">
              <el-tag :type="row.status === 'enabled' ? 'success' : 'danger'" size="small">
                {{ row.status === 'enabled' ? $t('common.enabled') : $t('common.disabled') }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column :label="$t('common.action')" width="140">
            <template #default="{ row }">
              <el-button type="primary" text size="small" @click="openEditItem(row)">{{ $t('common.edit') }}</el-button>
              <el-button type="danger" text size="small" @click="delItem(row)">{{ $t('common.delete') }}</el-button>
            </template>
          </el-table-column>
        </el-table>
      </div>
      <template #footer>
        <el-button @click="showItems = false">{{ $t('common.close') }}</el-button>
      </template>
    </OnDialog>

    <!-- 新增/编辑字典项 -->
    <OnDialog v-model="showItemForm" :title="itemForm.id ? $t('common.edit') : $t('common.add')" width="400px">
      <el-form :model="itemForm" label-width="100px">
        <el-form-item :label="$t('sys.dict.colLabel')" required>
          <el-input v-model="itemForm.label" />
        </el-form-item>
        <el-form-item :label="$t('sys.dict.colValue')" required>
          <el-input v-model="itemForm.value" />
        </el-form-item>
        <el-form-item :label="$t('sys.dict.colSort')">
          <el-input-number v-model="itemForm.sort" :min="0" />
        </el-form-item>
        <el-form-item :label="$t('common.status')">
          <el-radio-group v-model="itemForm.status">
            <el-radio-button value="enabled">{{ $t('common.enabled') }}</el-radio-button>
            <el-radio-button value="disabled">{{ $t('common.disabled') }}</el-radio-button>
          </el-radio-group>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showItemForm = false">{{ $t('common.cancel') }}</el-button>
        <el-button type="primary" @click="saveItem" :loading="saving">{{ $t('common.confirm') }}</el-button>
      </template>
    </OnDialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'
import { ElMessage } from 'element-plus'
import { useI18n } from 'vue-i18n'
import {
  listDicts,
  getDict,
  createDict,
  updateDict,
  deleteDict,
  createDictItem,
  updateDictItem,
  deleteDictItem,
} from '@/api/sys/dicts'
import type { Dict, DictItem } from '@/api/sys/dicts/type'
import { useMessage } from '@/hooks'

const { t } = useI18n()
const { confirm } = useMessage()

const dicts = ref<Dict[]>([])
const loading = ref(false)
const saving = ref(false)
const showForm = ref(false)
const form = reactive<Dict>({ name: '', code: '', description: '', status: 'enabled' })

async function load() {
  loading.value = true
  try {
    dicts.value = (await listDicts()) || []
  } catch (e: any) {
    ElMessage.error(e?.message || "common.fail")
  } finally {
    loading.value = false
  }
}

function openAdd() {
  Object.assign(form, { id: undefined, name: '', code: '', description: '', status: 'enabled' })
  showForm.value = true
}

function openEdit(row: Dict) {
  Object.assign(form, { id: row.id, name: row.name, code: row.code, description: row.description, status: row.status })
  showForm.value = true
}

async function save() {
  if (!form.name || !form.code) {
    ElMessage.warning(t('common.tip') + ': ' + t('sys.dict.colName') + ' / ' + t('sys.dict.colCode'))
    return
  }
  saving.value = true
  try {
    if (form.id) {
      await updateDict(form.id, { name: form.name, code: form.code, description: form.description, status: form.status })
    } else {
      await createDict({ name: form.name, code: form.code, description: form.description })
    }
    ElMessage.success(t('common.success'))
    showForm.value = false
    load()
  } catch (e: any) {
    ElMessage.error(e?.message || "common.fail")
  } finally {
    saving.value = false
  }
}

async function del(row: Dict) {
  const ok = await confirm({ message: "common.confirmDelete" })
  if (!ok) return
  try {
    await deleteDict(row.id!)
    ElMessage.success(t('common.success'))
    load()
  } catch (e: any) {
    ElMessage.error(e?.message || "common.fail")
  }
}

// ====== 字典项 ======
const showItems = ref(false)
const currentDict = ref<Dict | null>(null)
const items = ref<DictItem[]>([])
const loadingItems = ref(false)
const showItemForm = ref(false)
const itemForm = reactive<DictItem>({ label: '', value: '', sort: 0, status: 'enabled' })

async function openItems(row: Dict) {
  currentDict.value = row
  showItems.value = true
  await loadItems(row.id!)
}

async function loadItems(dictId: number) {
  loadingItems.value = true
  try {
    const data = await getDict(dictId)
    items.value = data.items || []
  } catch (e: any) {
    ElMessage.error(e?.message || "common.fail")
  } finally {
    loadingItems.value = false
  }
}

function openAddItem() {
  Object.assign(itemForm, { id: undefined, label: '', value: '', sort: 0, status: 'enabled' })
  showItemForm.value = true
}

function openEditItem(row: DictItem) {
  Object.assign(itemForm, { id: row.id, label: row.label, value: row.value, sort: row.sort, status: row.status })
  showItemForm.value = true
}

async function saveItem() {
  if (!itemForm.label || !itemForm.value) {
    ElMessage.warning(t('common.tip') + ': ' + t('sys.dict.colLabel') + ' / ' + t('sys.dict.colValue'))
    return
  }
  saving.value = true
  try {
    if (itemForm.id) {
      await updateDictItem(itemForm.id, itemForm)
    } else {
      await createDictItem(currentDict.value!.id!, itemForm)
    }
    ElMessage.success(t('common.success'))
    showItemForm.value = false
    loadItems(currentDict.value!.id!)
  } catch (e: any) {
    ElMessage.error(e?.message || "common.fail")
  } finally {
    saving.value = false
  }
}

async function delItem(row: DictItem) {
  const ok = await confirm({ message: "common.confirmDelete" })
  if (!ok) return
  try {
    await deleteDictItem(row.id!)
    ElMessage.success(t('common.success'))
    loadItems(currentDict.value!.id!)
  } catch (e: any) {
    ElMessage.error(e?.message || "common.fail")
  }
}

load()
</script>
