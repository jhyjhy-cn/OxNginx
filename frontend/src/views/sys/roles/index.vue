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
      >
        <template #toolbar-left>
          <el-button type="primary" @click="openCreate">{{ $t("common.add") }}</el-button>
        </template>
      </OnTable>
    </el-card>

    <!-- 创建弹窗 -->
    <OnDialog v-model="showCreate" :title="$t('sys.rbac.createRole')" width="400px">
      <OnForm ref="formRef" :model="form">
        <OnFormGrid :fields="formFields" :model="form" />
      </OnForm>
      <template #footer>
        <el-button @click="showCreate = false">{{ $t("common.cancel") }}</el-button>
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
import { ref, reactive, onMounted, nextTick } from "vue";
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
  deleteRole,
  getRoleMenus,
  setRoleMenus,
} from "@/api/sys/roles";
import { listMenus } from "@/api/sys/menus";

const { success, error, confirm } = useMessage();

const showCreate = ref(false);
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

const searchFields: FormField[] = [
  { prop: "keyword", label: "common.search", type: "input", span: 8 },
];

const isSuperAdmin = (row: any) => row.code === "super_admin";

const tableColumns: TableColumn[] = [
  { prop: "id", label: "sys.user.id", width: 60 },
  { prop: "code", label: "sys.rbac.colCode", width: 160 },
  { prop: "name", label: "sys.rbac.colName" },
  { prop: "remark", label: "sys.rbac.colRemark" },
  {
    label: "common.action",
    width: 200,
    buttons: [
      { name: "sys.rbac.menuPermission", command: "menuPerm", size: "small", disabled: isSuperAdmin },
      { name: "common.delete", command: "delete", type: "danger", size: "small", disabled: isSuperAdmin },
    ],
  },
];

const formFields: FormField[] = [
  { prop: "code", label: "sys.rbac.colCode", type: "input", required: true },
  { prop: "name", label: "sys.rbac.colName", type: "input", required: true },
  { prop: "remark", label: "sys.rbac.colRemark", type: "input" },
];

function onPageChange(p: number) {
  page.value = p;
  load();
}

function handleCommand(command: string | number, row: any) {
  if (command === "menuPerm") openMenuPerm(row);
  else if (command === "delete") del(row);
}

function openCreate() {
  Object.assign(form, { code: "", name: "", remark: "" });
  showCreate.value = true;
}

async function submit() {
  if (!formRef.value) return;
  try {
    await formRef.value.validate();
  } catch {
    return;
  }
  try {
    await createRole({ ...form });
    success("common.success");
    showCreate.value = false;
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
</style>
