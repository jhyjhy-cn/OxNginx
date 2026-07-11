<template>
  <div class="roles-page h-full">
    <el-card class="h-full">
      <!-- 搜索栏 -->
      <div class="search-bar">
        <OnFormGrid :model="searchForm" :fields="searchFields" style="flex: 1" />
        <el-button type="primary" @click="search">{{ $t("common.search") }}</el-button>
        <el-button @click="reset">{{ $t("common.reset") }}</el-button>
      </div>

      <!-- 表格 -->
      <OnTable
        :data="dataList"
        :columns="tableColumns"
        :loading="loading"
        :pagination="{ total, currentPage: page, pageSize }"
        :options="{ height: 'auto' }"
        @page-change="onPageChange"
        @command="handleCommand"
        @reload="load"
        @selectionChange="(rows: any[]) => (selectedRows = rows)"
      >
        <template #toolbar-left>
          <el-button v-auth="'sys:role:add'" type="primary" @click="openCreate">{{ $t("common.add") }}</el-button>
          <el-button
            v-auth="'sys:role:batchDelete'"
            type="danger"
            :disabled="!selectedRows.length"
            @click="batchDelete"
          >
            {{ $t("common.batchDelete") }} ({{ selectedRows.length }})
          </el-button>
        </template>
      </OnTable>
    </el-card>

    <!-- 创建/编辑弹窗 -->
    <OnDialog v-model="showForm" :title="$t(formTitle)" width="400px">
      <OnForm ref="formRef" :model="form">
        <OnFormGrid :fields="formFields" :model="form" />
      </OnForm>
      <template #footer>
        <el-button @click="showForm = false">{{ $t("common.cancel") }}</el-button>
        <el-button type="primary" @click="submit">{{ $t("common.confirm") }}</el-button>
      </template>
    </OnDialog>

    <!-- 菜单权限弹窗 -->
    <OnDialog
      v-model="showMenuPerm"
      :title="`${$t('sys.rbac.menuPermission')} - ${menuPermRole?.name || ''}`"
      width="520px"
      height="60vh"
      destroy-on-close
    >
      <div class="menu-perm-toolbar">
        <el-button size="small" @click="treeExpandAll(true)">{{ $t('common.expandAll') }}</el-button>
        <el-button size="small" @click="treeExpandAll(false)">{{ $t('common.collapseAll') }}</el-button>
        <el-button size="small" @click="treeSelectAll">{{ $t('common.selectAll') }}</el-button>
        <el-button size="small" @click="treeInvert">{{ $t('common.invert') }}</el-button>
        <el-button size="small" @click="treeClear">{{ $t('common.clear') }}</el-button>
      </div>
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
        <el-button @click="showMenuPerm = false">{{ $t("common.cancel") }}</el-button>
        <el-button type="primary" :loading="saving" @click="saveMenuPerm">{{ $t("common.save") }}</el-button>
      </template>
    </OnDialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, nextTick, computed } from "vue";
import OnForm from "@/components/OnForm/OnForm/index.vue";
import OnFormGrid from "@/components/OnForm/OnFormGrid/index.vue";
import OnDialog from "@/components/OnDialog/index.vue";
import type { FormField } from "@/components/OnForm/types";
import type { TableColumn } from "@/components/OnTable/types";
import OnTable from "@/components/OnTable/index.vue";
import { useCrud, useMessage } from "@/hooks";
import {
  listRoles,
  createRole,
  updateRole,
  deleteRole,
  batchDeleteRoles,
  getRoleMenus,
  setRoleMenus,
} from "@/api/sys/roles";
import { listMenus } from "@/api/sys/menus";

const showForm = ref(false);
const isEdit = ref(false);
const editingId = ref<number | null>(null);
const formRef = ref<InstanceType<typeof OnForm>>();
const form = reactive({ code: "", name: "", remark: "" });

const {
  loading,
  dataList,
  total,
  page,
  pageSize,
  searchForm,
  load,
  search,
  reset,
} = useCrud({
  getListApi: listRoles,
  isPage: true,
  pageSize: 20,
});
const { success, error, confirm } = useMessage();

const selectedRows = ref<any[]>([]);

const searchFields: FormField[] = [
  { prop: "keyword", label: "common.search", type: "input", span: 8 },
];

const isSuperAdmin = (row: any) => row.code === "super_admin";

const tableColumns: TableColumn[] = [
  { type: "selection", width: 48 },
  { prop: "code", label: "sys.rbac.colCode", width: 160 },
  { prop: "name", label: "sys.rbac.colName" },
  { prop: "remark", label: "sys.rbac.colRemark" },
  {
    label: "common.action",
    width: 240,
    buttons: [
      { name: "common.edit", command: "edit", size: "small" },
      { name: "sys.rbac.menuPermission", command: "menuPerm", size: "small", disabled: isSuperAdmin },
      { name: "common.delete", command: "delete", type: "danger", size: "small", disabled: isSuperAdmin },
    ],
  },
];

const formFields = computed<FormField[]>(() => [
  { prop: "code", label: "sys.rbac.colCode", type: "input", required: true, disabled: isEdit.value },
  { prop: "name", label: "sys.rbac.colName", type: "input", required: true },
  { prop: "remark", label: "sys.rbac.colRemark", type: "input" },
]);

// ponytail: template 里 $t() 取 i18n,script setup 里 useI18n().t —— 这里只返回 i18n key 让模板翻译
const formTitle = computed(() => (isEdit.value ? "common.edit" : "sys.rbac.createRole"));

function onPageChange(p: number) {
  page.value = p;
  load();
}

function handleCommand(command: string | number, row: any) {
  if (command === "edit") openEdit(row);
  else if (command === "menuPerm") openMenuPerm(row);
  else if (command === "delete") del(row);
}

function openCreate() {
  isEdit.value = false;
  editingId.value = null;
  Object.assign(form, { code: "", name: "", remark: "" });
  showForm.value = true;
}

async function openEdit(row: any) {
  isEdit.value = true;
  editingId.value = row.id;
  Object.assign(form, { code: row.code || "", name: row.name || "", remark: row.remark || "" });
  showForm.value = true;
}

async function submit() {
  if (!formRef.value) return;
  try {
    await formRef.value.validate();
  } catch {
    return;
  }
  try {
    if (isEdit.value && editingId.value != null) {
      await updateRole(editingId.value, { ...form });
    } else {
      await createRole({ ...form });
    }
    success("common.success");
    showForm.value = false;
    load();
  } catch (e: any) {
    error(e?.message || "common.fail");
  }
}

async function del(row: any) {
  const ok = await confirm({ message: "common.confirmDelete" });
  if (!ok) return;
  try {
    await deleteRole(row.id);
    success("common.success");
    load();
  } catch (e: any) {
    error(e?.message || "common.fail");
  }
}

async function batchDelete() {
  if (!selectedRows.value.length) return;
  const supers = selectedRows.value.filter(isSuperAdmin);
  if (supers.length) {
    error("sys.rbac.adminCannotDelete");
    return;
  }
  const ok = await confirm({
    message: "sys.rbac.confirmBatchDelete",
    params: { n: selectedRows.value.length },
  });
  if (!ok) return;
  try {
    const ids = selectedRows.value.map((r) => r.id);
    const msg = await batchDeleteRoles(ids);
    success(typeof msg === "string" ? msg : "common.success");
    selectedRows.value = [];
    load();
  } catch (e: any) {
    error(e?.message || "common.fail");
  }
}

// ========== 菜单权限弹窗 ==========
const showMenuPerm = ref(false);
const menuPermRole = ref<any>(null);
const menuTree = ref<any[]>([]);
const checkedMenuIds = ref<number[]>([]);
const treeRef = ref();
const saving = ref(false);

async function openMenuPerm(row: any) {
  menuPermRole.value = row;
  showMenuPerm.value = true;
  checkedMenuIds.value = [];
  const [menus, ids] = await Promise.all([
    listMenus({ page: 1, page_size: 999 }),
    getRoleMenus(row.id),
  ]);
  const list: any[] = menus || [];
  const map = new Map<number, any>();
  list.forEach((m) => map.set(m.id, { ...m, children: [] as any[] }));
  const roots: any[] = [];
  for (const m of map.values()) {
    if (m.parent_id && map.has(m.parent_id)) map.get(m.parent_id).children.push(m);
    else roots.push(m);
  }
  menuTree.value = roots;
  checkedMenuIds.value = (ids || []) as number[];
  await nextTick();
  if (treeRef.value && checkedMenuIds.value.length) {
    treeRef.value.setCheckedKeys(checkedMenuIds.value);
  }
}

async function saveMenuPerm() {
  if (!menuPermRole.value) return;
  if (!treeRef.value) {
    error("tree not ready");
    return;
  }
  saving.value = true;
  try {
    const checked = treeRef.value.getCheckedKeys() as number[];
    const half = treeRef.value.getHalfCheckedKeys() as number[];
    const ids = [...checked, ...half];
    await setRoleMenus(menuPermRole.value.id, ids);
    success("common.success");
    showMenuPerm.value = false;
  } catch (e: any) {
    error(e?.message || "common.fail");
  } finally {
    saving.value = false;
  }
}

// ====== 树快捷操作 ======
const flattenIds = (nodes: any[]): number[] => {
  const ids: number[] = [];
  const walk = (ns: any[]) => {
    for (const n of ns) {
      if (n.id) ids.push(n.id);
      if (n.children?.length) walk(n.children);
    }
  };
  walk(nodes);
  return ids;
};

function treeExpandAll(expand: boolean) {
  const ids = flattenIds(menuTree.value);
  ids.forEach((id) => treeRef.value?.store?.nodesMap[id]?.expand()); // ponytail: expand 全部走 store;传 false 用 collapse
  if (!expand) ids.forEach((id) => treeRef.value?.store?.nodesMap[id]?.collapse());
}

function treeSelectAll() {
  const ids = flattenIds(menuTree.value);
  treeRef.value?.setCheckedKeys(ids);
}

function treeInvert() {
  if (!treeRef.value) return;
  const all = flattenIds(menuTree.value);
  const checked = new Set<number>(
    treeRef.value.getCheckedKeys() as number[],
  );
  const half = new Set<number>(treeRef.value.getHalfCheckedKeys() as number[]);
  // ponytail: 当前已勾的（包括半选父）取反
  const nextChecked = all.filter((id) => !checked.has(id) && !half.has(id));
  const nextUncheck = new Set([...checked, ...half]);
  treeRef.value.setCheckedKeys(nextChecked);
  // ponytail: 那些"曾经选中"的节点若不在新选中集里,要显式清掉,否则它们的子树会保留
  for (const id of nextUncheck) {
    if (!nextChecked.includes(id)) treeRef.value.setChecked(id, false, false);
  }
}

function treeClear() {
  treeRef.value?.setCheckedKeys([]);
}

onMounted(() => {
  load();
});
</script>

<style scoped>
.search-bar {
  display: flex;
  gap: 12px;
  align-items: flex-start;
  margin-bottom: 12px;
}
.menu-perm-toolbar {
  display: flex;
  gap: 8px;
  margin-bottom: 8px;
  flex-wrap: wrap;
}
</style>
