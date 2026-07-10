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
        <el-button type="primary" @click="showCreate = true">{{ $t('common.add') }}</el-button>
      </div>

      <el-table :data="roles" v-loading="loading" max-height="calc(100vh - 340px)">
        <el-table-column prop="id" label="ID" width="60" />
        <el-table-column prop="code" :label="$t('sys.rbac.colCode')" width="160" />
        <el-table-column prop="name" :label="$t('sys.rbac.colName')" />
        <el-table-column prop="remark" :label="$t('sys.rbac.colRemark')" />
        <el-table-column :label="$t('common.action')" width="200">
          <template #default="{ row }">
            <el-button size="small" :disabled="row.code === 'super_admin'" @click="openMenuPerm(row)">
              {{ $t('sys.rbac.menuPermission') }}
            </el-button>
            <el-button size="small" type="danger" :disabled="row.code === 'super_admin'" @click="del(row)">
              {{ $t('common.delete') }}
            </el-button>
          </template>
        </el-table-column>
      </el-table>

      <OnPagination v-model:current-page="currentPage" v-model:page-size="pageSize" :total="total" @change="load" />
    </el-card>

    <el-dialog v-model="showCreate" :title="$t('sys.rbac.createRole')" width="400px">
      <el-form :model="form" label-width="80px">
        <el-form-item :label="$t('sys.rbac.colCode')"><el-input v-model="form.code" /></el-form-item>
        <el-form-item :label="$t('sys.rbac.colName')"><el-input v-model="form.name" /></el-form-item>
        <el-form-item :label="$t('sys.rbac.colRemark')"><el-input v-model="form.remark" /></el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showCreate = false">{{ $t('common.cancel') }}</el-button>
        <el-button type="primary" @click="submit">{{ $t('common.confirm') }}</el-button>
      </template>
    </el-dialog>

    <!-- 菜单权限弹窗 -->
    <OnDialog
      v-model="showMenuPerm"
      :title="`${$t('sys.rbac.menuPermission')} - ${menuPermRole?.name || ''}`"
      width="520px"
      height="60vh"
      destroy-on-close
    >
      <el-tree
        ref="treeRef"
        :data="menuTree"
        show-checkbox
        node-key="id"
        :default-checked-keys="checkedMenuIds"
        :props="{ label: (data: any) => $t(data.title), children: 'children' }"
        style="margin-top: 4px"
      />
      <template #footer>
        <el-button @click="showMenuPerm = false">{{ $t('common.cancel') }}</el-button>
        <el-button type="primary" :loading="saving" @click="saveMenuPerm">{{ $t('common.save') }}</el-button>
      </template>
    </OnDialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, nextTick } from 'vue'
import OnPagination from '@/components/OnPagination/index.vue'
import OnDialog from '@/components/OnDialog/index.vue'
import {
  listRoles,
  createRole,
  deleteRole,
  getRoleMenus,
  setRoleMenus,
} from '@/api/sys/roles'
import { listMenus } from '@/api/sys/menus'
import { useMessage } from '@/hooks'

const { success, error, confirm } = useMessage()
const roles = ref<any[]>([])
const loading = ref(false)
const showCreate = ref(false)
const form = reactive({ code: '', name: '', remark: '' })
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
    const data = await listRoles(params)
    roles.value = data.list || []
    total.value = data.total || 0
  } catch (e: any) {
    error(e?.message || "common.fail")
  } finally {
    loading.value = false
  }
}

async function submit() {
  if (!form.code || !form.name) return
  try {
    await createRole(form)
    success("common.success")
    showCreate.value = false
    form.code = ''
    form.name = ''
    form.remark = ''
    load()
  } catch (e: any) {
    error(e?.message || "common.fail")
  }
}

async function del(row: any) {
  const ok = await confirm({ message: "common.confirmDelete" })
  if (!ok) return
  try {
    await deleteRole(row.id)
    success("common.success")
    load()
  } catch (e: any) {
    error(e?.message || "common.fail")
  }
}

// ========== 菜单权限弹窗 ==========
const showMenuPerm = ref(false)
const menuPermRole = ref<any>(null)
const menuTree = ref<any[]>([])
const checkedMenuIds = ref<number[]>([])
const treeRef = ref()
const saving = ref(false)

async function openMenuPerm(row: any) {
  menuPermRole.value = row
  showMenuPerm.value = true
  checkedMenuIds.value = []
  const [menus, ids] = await Promise.all([
    listMenus({ page: 1, page_size: 999 }),
    getRoleMenus(row.id),
  ])
  const list: any[] = menus || []
  const map = new Map<number, any>()
  list.forEach((m) => map.set(m.id, { ...m, children: [] as any[] }))
  const roots: any[] = []
  for (const m of map.values()) {
    if (m.parent_id && map.has(m.parent_id)) map.get(m.parent_id).children.push(m)
    else roots.push(m)
  }
  menuTree.value = roots
  checkedMenuIds.value = (ids || []) as number[]
  await nextTick()
  if (treeRef.value && checkedMenuIds.value.length) {
    treeRef.value.setCheckedKeys(checkedMenuIds.value)
  }
}

async function saveMenuPerm() {
  if (!menuPermRole.value) return
  if (!treeRef.value) {
    error('tree not ready')
    return
  }
  saving.value = true
  try {
    const checked = treeRef.value.getCheckedKeys() as number[]
    const half = treeRef.value.getHalfCheckedKeys() as number[]
    const ids = [...checked, ...half]
    await setRoleMenus(menuPermRole.value.id, ids)
    success("common.success")
    showMenuPerm.value = false
  } catch (e: any) {
    error(e?.message || "common.fail")
  } finally {
    saving.value = false
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
