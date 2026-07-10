<template>
  <div class="users-page h-full">
    <el-card class="h-full">
      <div class="flex h-full">
        <!-- 左侧：部门树 -->
        <div class="dept-tree">
          <div class="dept-tree__header">
            <span>{{ $t("rbac.department") }}</span>
          </div>
          <el-tree
            :data="deptTree"
            :props="{ children: 'children', label: 'name' }"
            node-key="id"
            :expand-on-click-node="false"
            :default-expand-all="true"
            @node-click="handleDeptNodeClick"
          >
            <template #default="{ data }">
              <span class="dept-node">
                <span>{{ data.name }}</span>
              </span>
            </template>
          </el-tree>
        </div>

        <!-- 右侧：搜索 + 表格 -->
        <div class="main-area">
          <!-- 搜索栏 -->
          <div class="search-bar">
            <OnFormGrid
              :model="searchForm"
              :fields="searchFields"
              style="flex: 1"
            />
            <el-button type="primary" @click="search">{{
              $t("common.search")
            }}</el-button>
            <el-button @click="reset">{{ $t("common.reset") }}</el-button>
          </div>

          <!-- 表格 -->
          <OnTable
            ref="tableRef"
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
              <el-button type="primary" @click="openCreate">
                {{ $t("common.add") }}
              </el-button>
              <el-button
                type="warning"
                :disabled="!selectedRows.length"
                @click="batchResetPwd"
              >
                {{ $t("rbac.resetPassword") }} ({{ selectedRows.length }})
              </el-button>
              <el-dropdown
                :disabled="!selectedRows.length"
                @command="batchSetDisabled"
              >
                <el-button type="success" :disabled="!selectedRows.length">
                  {{ $t("common.status") }} ({{ selectedRows.length }})
                  <el-icon style="margin-left: 4px"><ArrowDown /></el-icon>
                </el-button>
                <template #dropdown>
                  <el-dropdown-menu>
                    <el-dropdown-item :command="1">{{
                      $t("common.disabled")
                    }}</el-dropdown-item>
                    <el-dropdown-item :command="0">{{
                      $t("common.enabled")
                    }}</el-dropdown-item>
                  </el-dropdown-menu>
                </template>
              </el-dropdown>
              <el-button @click="exportXlsx">
                <el-icon style="margin-right: 4px"><Download /></el-icon>
                {{ $t("common.export") }}
              </el-button>
            </template>
            <template #status="{ row }">
              <el-tag :type="row.disabled ? 'danger' : 'success'" size="small">
                {{
                  row.disabled ? $t("common.disabled") : $t("common.enabled")
                }}
              </el-tag>
            </template>
            <template #gender="{ row }">
              {{ genderLabels[row.gender] || "-" }}
            </template>
          </OnTable>
        </div>
      </div>
    </el-card>

    <!-- 创建/编辑弹窗 -->
    <OnDialog v-model="showForm" :title="formTitle" width="600px">
      <OnForm ref="formRef" :model="form">
        <OnFormGrid :fields="formFields" :model="form" />
      </OnForm>
      <template #footer>
        <el-button @click="showForm = false">{{
          $t("common.cancel")
        }}</el-button>
        <el-button type="primary" @click="submit">{{
          $t("common.confirm")
        }}</el-button>
      </template>
    </OnDialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, computed } from "vue";
import { ArrowDown, Download } from "@element-plus/icons-vue";
import api from "@/api";
import OnForm from "@/components/OnForm/OnForm/index.vue";
import OnFormGrid from "@/components/OnForm/OnFormGrid/index.vue";
import OnDialog from "@/components/OnDialog/index.vue";
import type { FormField } from "@/components/OnForm/types";
import type { TableColumn } from "@/components/OnTable/types";
import OnTable from "@/components/OnTable/index.vue";
import { useCrud, useMessage } from "@/hooks";
import { downloadXlsx } from "@/utils/export";

const { confirm, success, error } = useMessage();

// 已选行
const selectedRows = ref<any[]>([]);

// 部门树
const deptTree = ref<any[]>([]);
const selectedDeptId = ref<number | null>(null);

async function loadDeptTree() {
  try {
    const { data } = await api.get("/api/rbac/depts/tree");
    if (data.code === 0) deptTree.value = data.data || [];
  } catch {}
}

function handleDeptNodeClick(node: any) {
  selectedDeptId.value = node.id;
  searchForm.dept_id = node.id;
  load();
}

// CRUD
const showForm = ref(false);
const isEdit = ref(false);
const editingId = ref<number | null>(null);
const formRef = ref<InstanceType<typeof OnForm>>();

const form = reactive({
  nickname: "",
  dept_id: null as number | null,
  phone: "",
  email: "",
  username: "",
  password: "",
  gender: "secret",
  disabled: 0,
  post_id: null as number | null,
  role_ids: [] as number[],
  remark: "",
});

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
  getListApi: (params) =>
    api.get("/api/rbac/users", { params }).then((r) => r.data),
  isPage: true,
  pageSize: 20,
});

// 搜索表单
searchForm.dept_id = null;

// 性别选项
const genderOptions = [
  { label: "男", value: "male" },
  { label: "女", value: "female" },
  { label: "保密", value: "secret" },
];

const genderLabels: Record<string, string> = {
  male: "男",
  female: "女",
  secret: "保密",
};

// 岗位列表
const posts = ref<any[]>([]);
async function loadPosts() {
  try {
    const { data } = await api.get("/api/rbac/posts", {
      params: { page_size: 100 },
    });
    if (data.code === 0) posts.value = data.data?.list || [];
  } catch {}
}

// 角色列表
const roles = ref<any[]>([]);
async function loadRoles() {
  try {
    const { data } = await api.get("/api/rbac/roles", {
      params: { page_size: 100 },
    });
    if (data.code === 0) roles.value = data.data?.list || [];
  } catch {}
}

// 搜索栏字段
const searchFields: FormField[] = [
  { prop: "username", label: "login.username", type: "input", span: 6 },
  { prop: "phone", label: "user.phone", type: "input", span: 6 },
  {
    prop: "status",
    label: "common.status",
    type: "select",
    span: 6,
    options: [
      { label: "common.all", value: "" },
      { label: "common.enabled", value: "enabled" },
      { label: "common.disabled", value: "disabled" },
    ],
  },
];

// 表格列
const tableColumns: TableColumn[] = [
  { type: "selection", width: 48 },
  { prop: "id", label: "user.id", width: 70 },
  { prop: "username", label: "login.username" },
  { prop: "nickname", label: "user.nickname" },
  { prop: "dept_name", label: "rbac.department" },
  { prop: "phone", label: "user.phone" },
  { prop: "gender", label: "user.gender", width: 80, slot: "gender" },
  { prop: "disabled", label: "common.status", width: 80, slot: "status" },
  { prop: "created_at", label: "common.createdAt", width: 170 },
  {
    label: "common.action",
    width: 200,
    buttons: [
      { name: "common.edit", command: "edit", size: "small" },
      { name: "rbac.resetPassword", command: "resetPwd", size: "small" },
      {
        name: "common.delete",
        command: "delete",
        type: "danger",
        size: "small",
        disabled: (row: any) => row.username === "admin",
      },
    ],
  },
];

// 部门选项（扁平）
const deptOptions = computed<FormField["options"]>(() => {
  const result: FormField["options"] = [];
  const flatten = (nodes: any[]) => {
    for (const n of nodes) {
      if (n.id) result.push({ label: n.name, value: n.id });
      if (n.children?.length) flatten(n.children);
    }
  };
  flatten(deptTree.value);
  return result;
});

// 岗位选项
const postOptions = computed(() =>
  posts.value.map((p: any) => ({ label: p.name, value: p.id })),
);

// 角色选项
const roleOptions = computed(() =>
  roles.value.map((r: any) => ({ label: r.name, value: r.id })),
);

// 表单字段
const formFields = computed<FormField[]>(() => {
  const fields: FormField[] = [
    {
      prop: "username",
      label: "login.username",
      type: "input",
      required: true,
    },
  ];
  if (!isEdit.value) {
    fields.push({
      prop: "password",
      label: "login.password",
      type: "password",
      required: true,
      showPassword: true,
    });
  }
  fields.push(
    { prop: "nickname", label: "user.nickname", type: "input" },
    {
      prop: "dept_id",
      label: "rbac.department",
      type: "select",
      required: true,
      options: deptOptions.value,
    },
    { prop: "phone", label: "user.phone", type: "input" },
    { prop: "email", label: "user.email", type: "input" },
    {
      prop: "gender",
      label: "user.gender",
      type: "radio",
      options: genderOptions,
    },
    { prop: "disabled", label: "common.status", type: "switch" },
    {
      prop: "post_id",
      label: "rbac.post",
      type: "select",
      options: postOptions.value,
    },
    {
      prop: "role_ids",
      label: "rbac.roles",
      type: "select",
      multiple: true,
      required: true,
      rules: [{
        validator: (_: any, v: number[], cb: (e?: Error) => void) =>
          v?.length ? cb() : cb(new Error("请选择角色")),
        trigger: "change",
      }],
      options: roleOptions.value,
    },
    { prop: "remark", label: "common.remark", type: "textarea" },
  );
  return fields;
});

const formTitle = computed(() =>
  isEdit.value ? "common.edit" : "rbac.createUser",
);

// 翻页
function onPageChange(p: number) {
  page.value = p;
  load();
}

// 操作
function handleCommand(command: string | number, row: any) {
  if (command === "edit") openEdit(row);
  else if (command === "resetPwd") resetPwd(row);
  else if (command === "delete") del(row);
}

function openCreate() {
  isEdit.value = false;
  editingId.value = null;
  Object.assign(form, {
    nickname: "",
    dept_id: null,
    phone: "",
    email: "",
    username: "",
    password: "",
    gender: "secret",
    disabled: 0,
    post_id: null,
    role_ids: [],
    remark: "",
  });
  showForm.value = true;
}

async function openEdit(row: any) {
  isEdit.value = true;
  editingId.value = row.id;
  try {
    const { data } = await api.get(`/api/rbac/users/${row.id}`);
    if (data.code === 0) {
      Object.assign(form, {
        username: data.data.username || "",
        nickname: data.data.nickname || "",
        dept_id: data.data.dept_id,
        phone: data.data.phone || "",
        email: data.data.email || "",
        gender: data.data.gender || "secret",
        disabled: data.data.disabled,
        post_id: data.data.post_id,
        role_ids: data.data.role_ids || [],
        remark: data.data.remark || "",
      });
      showForm.value = true;
    }
  } catch {}
}

async function submit() {
  if (!formRef.value) return;
  try {
    await formRef.value.validate();
  } catch {
    return; // el-form 校验失败会自动红字提示
  }

  const payload: Record<string, any> = {
    username: form.username,
    nickname: form.nickname || undefined,
    dept_id: form.dept_id || undefined,
    phone: form.phone || undefined,
    email: form.email || undefined,
    gender: form.gender || undefined,
    disabled: form.disabled,
    post_id: form.post_id || undefined,
    role_ids: form.role_ids.length ? form.role_ids : undefined,
    remark: form.remark || undefined,
  };
  if (!isEdit.value) payload.password = form.password;

  try {
    const { data } = isEdit.value
      ? await api.put(`/api/rbac/users/${editingId.value}`, payload)
      : await api.post("/api/rbac/users", payload);
    if (data.code === 0) {
      success(isEdit.value ? "rbac.userUpdateSuccess" : "rbac.userCreateSuccess");
      showForm.value = false;
      load();
    } else {
      error(data.message);
    }
  } catch {}
}

async function resetPwd(row: any) {
  const ok = await confirm({ message: "common.confirmResetPwd" });
  if (!ok) return;
  try {
    const { data } = await api.post(
      `/api/rbac/users/${row.id}/reset-password`,
      { new_password: "123456" },
    );
    if (data.code === 0) success("common.success");
    else error(data.message);
  } catch (e: any) {
    error(e?.data?.message || e?.message || "common.fail");
  }
}

async function del(row: any) {
  const ok = await confirm({ message: "common.confirmDelete" });
  if (!ok) return;
  try {
    const { data } = await api.delete(`/api/rbac/users/${row.id}`);
    if (data.code === 0) {
      success("common.deleteSuccess");
      load();
    } else error(data.message);
  } catch {}
}

// 批量操作
async function batchResetPwd() {
  if (!selectedRows.value.length) return;
  const adminCount = selectedRows.value.filter((r) => r.username === "admin").length;
  if (adminCount > 0) {
    error("rbac.adminCannotReset");
    return;
  }
  const ok = await confirm({ message: "rbac.confirmBatchResetPwd" });
  if (!ok) return;
  try {
    const ids = selectedRows.value.map((r) => r.id);
    const { data } = await api.post("/api/rbac/users/batch/reset-password", { ids });
    if (data.code === 0) {
      success(data.data);
      selectedRows.value = [];
    } else error(data.message);
  } catch (e: any) {
    error(e?.response?.data?.message || e?.message || "common.fail");
  }
}

async function batchSetDisabled(disabled: number) {
  if (!selectedRows.value.length) return;
  const adminCount = selectedRows.value.filter((r) => r.username === "admin").length;
  if (adminCount > 0 && disabled === 1) {
    error("rbac.adminCannotDisable");
    return;
  }
  const ok = await confirm({
    message: disabled === 1 ? "rbac.confirmBatchDisable" : "rbac.confirmBatchEnable",
  });
  if (!ok) return;
  try {
    const ids = selectedRows.value.map((r) => r.id);
    const { data } = await api.post("/api/rbac/users/batch/disabled", { ids, disabled });
    if (data.code === 0) {
      success(data.data);
      selectedRows.value = [];
      load();
    } else error(data.message);
  } catch (e: any) {
    error(e?.response?.data?.message || e?.message || "common.fail");
  }
}

// 导出 xlsx（按当前查询条件）
async function exportXlsx() {
  const params: Record<string, any> = {
    page_size: 10000, // 借 page_size 当 limit
    ...searchForm,
  };
  Object.keys(params).forEach((k) => {
    if (params[k] === null || params[k] === undefined || params[k] === "") {
      delete params[k];
    }
  });
  const ts = new Date().toISOString().slice(0, 10);
  await downloadXlsx("/api/rbac/users/export", params, `users-${ts}.xlsx`);
}

onMounted(() => {
  loadDeptTree();
  loadPosts();
  loadRoles();
  load();
});
</script>

<style scoped>
.dept-tree {
  width: 220px;
  flex-shrink: 0;
  margin-right: 16px;
  border-right: 1px solid var(--el-border-color);
  padding-right: 12px;
}

.dept-tree__header {
  font-weight: 600;
  margin-bottom: 8px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--el-border-color);
}

.dept-node {
  display: flex;
  justify-content: space-between;
  width: 100%;
}

.main-area {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.search-bar {
  display: flex;
  gap: 12px;
  align-items: flex-start;
  margin-bottom: 12px;
}
</style>
