<template>
  <div class="dicts-page h-full">
    <el-card class="h-full">
      <OnTable
        :data="dataList"
        :columns="dictColumns"
        :loading="loading"
        :pagination="false"
        :options="{ height: 'auto' }"
        @command="handleDictCommand"
        @reload="load"
      >
        <template #toolbar-left>
          <el-button v-auth="'sys:dict:add'" type="primary" @click="openAdd">{{ $t('common.add') }}</el-button>
        </template>
        <template #status="{ row }">
          <el-tag :type="row.status === 'enabled' ? 'success' : 'danger'" size="small">
            {{ row.status === 'enabled' ? $t('common.enabled') : $t('common.disabled') }}
          </el-tag>
        </template>
      </OnTable>
    </el-card>

    <!-- 新增/编辑字典 -->
    <OnDialog v-model="showForm" :title="form.id ? $t('common.edit') : $t('common.add')" width="480px">
      <OnForm ref="formRef" :model="form">
        <OnFormGrid :fields="dictFields" :model="form" />
      </OnForm>
      <template #footer>
        <el-button @click="showForm = false">{{ $t('common.cancel') }}</el-button>
        <el-button type="primary" @click="save" :loading="saving">{{ $t('common.confirm') }}</el-button>
      </template>
    </OnDialog>

    <!-- 字典项管理 -->
    <OnDialog v-model="showItems" :title="$t('sys.dict.manageItems') + ' - ' + currentDict?.name" width="800px">
      <div v-if="currentDict" class="dict-items">
        <OnTable
          :data="items"
          :columns="itemColumns"
          :loading="loadingItems"
          :pagination="false"
          :options="{ maxHeight: 400 }"
          @command="handleItemCommand"
        >
          <template #toolbar-left>
            <el-button type="primary" size="small" @click="openAddItem">{{ $t('common.add') }}</el-button>
          </template>
          <template #status="{ row }">
            <el-tag :type="row.status === 'enabled' ? 'success' : 'danger'" size="small">
              {{ row.status === 'enabled' ? $t('common.enabled') : $t('common.disabled') }}
            </el-tag>
          </template>
        </OnTable>
      </div>
      <template #footer>
        <el-button @click="showItems = false">{{ $t('common.close') }}</el-button>
      </template>
    </OnDialog>

    <!-- 新增/编辑字典项 -->
    <OnDialog v-model="showItemForm" :title="itemForm.id ? $t('common.edit') : $t('common.add')" width="400px">
      <OnForm ref="itemFormRef" :model="itemForm">
        <OnFormGrid :fields="itemFields" :model="itemForm" />
      </OnForm>
      <template #footer>
        <el-button @click="showItemForm = false">{{ $t('common.cancel') }}</el-button>
        <el-button type="primary" @click="saveItem" :loading="saving">{{ $t('common.confirm') }}</el-button>
      </template>
    </OnDialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { useI18n } from 'vue-i18n'
import OnForm from '@/components/OnForm/OnForm/index.vue'
import OnFormGrid from '@/components/OnForm/OnFormGrid/index.vue'
import OnDialog from '@/components/OnDialog/index.vue'
import OnTable from '@/components/OnTable/index.vue'
import type { FormField } from '@/components/OnForm/types'
import type { TableColumn } from '@/components/OnTable/types'
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
import { useCrud, useMessage } from '@/hooks'

const { t } = useI18n()
const { confirm } = useMessage()

const saving = ref(false)
const showForm = ref(false)
const formRef = ref<InstanceType<typeof OnForm>>()
const form = reactive<Dict>({ name: '', code: '', remark: '', status: 'enabled' })

// ponytail: 字典接口返回整表，无分页
const { loading, dataList, load } = useCrud({
  getListApi: listDicts,
  isPage: false,
})

const dictColumns: TableColumn[] = [
  { prop: 'name', label: 'sys.dict.colName', minWidth: 160 },
  { prop: 'code', label: 'sys.dict.colCode', minWidth: 160 },
  { prop: 'remark', label: 'sys.dict.colDesc', minWidth: 200 },
  { prop: 'status', label: 'common.status', width: 100, slot: 'status' },
  {
    label: 'common.action',
    width: 220,
    fixed: 'right',
    buttons: [
      { name: 'common.edit', command: 'edit', size: 'small' },
      { name: 'sys.dict.manageItems', command: 'items', size: 'small' },
      { name: 'common.delete', command: 'delete', type: 'danger', size: 'small' },
    ],
  },
]

const dictFields = computed<FormField[]>(() => [
  { prop: 'name', label: 'sys.dict.colName', type: 'input', required: true },
  { prop: 'code', label: 'sys.dict.colCode', type: 'input', required: true, disabled: !!form.id },
  { prop: 'remark', label: 'sys.dict.colDesc', type: 'textarea', rows: 2 },
])

function handleDictCommand(command: string | number, row: Dict) {
  if (command === 'edit') openEdit(row)
  else if (command === 'items') openItems(row)
  else if (command === 'delete') del(row)
}

function openAdd() {
  Object.assign(form, { id: undefined, name: '', code: '', remark: '', status: 'enabled' })
  showForm.value = true
}

function openEdit(row: Dict) {
  Object.assign(form, { id: row.id, name: row.name, code: row.code, remark: row.remark, status: row.status })
  showForm.value = true
}

async function save() {
  if (!formRef.value) return
  try {
    await formRef.value.validate()
  } catch {
    return
  }
  saving.value = true
  try {
    if (form.id) {
      await updateDict(form.id, { name: form.name, code: form.code, remark: form.remark, status: form.status })
    } else {
      await createDict({ name: form.name, code: form.code, remark: form.remark })
    }
    ElMessage.success(t('common.success'))
    showForm.value = false
    load()
  } catch (e: any) {
    ElMessage.error(e?.message || 'common.fail')
  } finally {
    saving.value = false
  }
}

async function del(row: Dict) {
  const ok = await confirm({ message: 'common.confirmDelete' })
  if (!ok) return
  try {
    await deleteDict(row.id!)
    ElMessage.success(t('common.success'))
    load()
  } catch (e: any) {
    ElMessage.error(e?.message || 'common.fail')
  }
}

// ====== 字典项 ======
const showItems = ref(false)
const currentDict = ref<Dict | null>(null)
const items = ref<DictItem[]>([])
const loadingItems = ref(false)
const showItemForm = ref(false)
const itemFormRef = ref<InstanceType<typeof OnForm>>()
const itemForm = reactive<DictItem>({ label: '', value: '', sort: 0, status: 'enabled' })

const itemColumns: TableColumn[] = [
  { prop: 'label', label: 'sys.dict.colLabel', minWidth: 160 },
  { prop: 'value', label: 'sys.dict.colValue', minWidth: 160 },
  { prop: 'sort', label: 'sys.dict.colSort', width: 80 },
  { prop: 'status', label: 'common.status', width: 100, slot: 'status' },
  {
    label: 'common.action',
    width: 140,
    buttons: [
      { name: 'common.edit', command: 'edit', size: 'small' },
      { name: 'common.delete', command: 'delete', type: 'danger', size: 'small' },
    ],
  },
]

const itemFields: FormField[] = [
  { prop: 'label', label: 'sys.dict.colLabel', type: 'input', required: true },
  { prop: 'value', label: 'sys.dict.colValue', type: 'input', required: true },
  { prop: 'sort', label: 'sys.dict.colSort', type: 'number', min: 0 },
  {
    prop: 'status',
    label: 'common.status',
    type: 'radio',
    options: [
      { label: 'common.enabled', value: 'enabled' },
      { label: 'common.disabled', value: 'disabled' },
    ],
  },
]

function handleItemCommand(command: string | number, row: DictItem) {
  if (command === 'edit') openEditItem(row)
  else if (command === 'delete') delItem(row)
}

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
    ElMessage.error(e?.message || 'common.fail')
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
  if (!itemFormRef.value) return
  try {
    await itemFormRef.value.validate()
  } catch {
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
    ElMessage.error(e?.message || 'common.fail')
  } finally {
    saving.value = false
  }
}

async function delItem(row: DictItem) {
  const ok = await confirm({ message: 'common.confirmDelete' })
  if (!ok) return
  try {
    await deleteDictItem(row.id!)
    ElMessage.success(t('common.success'))
    loadItems(currentDict.value!.id!)
  } catch (e: any) {
    ElMessage.error(e?.message || 'common.fail')
  }
}

onMounted(() => {
  load()
})
</script>
