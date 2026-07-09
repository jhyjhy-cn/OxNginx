<template>
  <div class="rbac-page h-full">
    <el-card
      class="h-full"
      body-style="height: 100%; display: flex; flex-direction: column; overflow: hidden;"
    >
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
      >
        <template #toolbar-left>
          <el-button type="primary" @click="showCreate = true">{{
            $t("common.add")
          }}</el-button>
        </template>
        <template #status="{ row }">
          <el-tag :type="row.disabled ? 'danger' : 'success'" size="small">
            {{ row.disabled ? $t("common.disabled") : $t("common.enabled") }}
          </el-tag>
        </template>
      </OnTable>
    </el-card>

    <OnDialog v-model="showCreate" :title="$t('rbac.createUser')" width="400px">
      <OnForm :model="form">
        <OnFormGrid :fields="formFields" :model="form" />
      </OnForm>
      <template #footer>
        <el-button @click="showCreate = false">{{
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
import { ref, reactive, onMounted } from "vue";
import api from "@/api";
import OnForm from "@/components/OnForm/OnForm/index.vue";
import OnFormGrid from "@/components/OnForm/OnFormGrid/index.vue";
import OnDialog from "@/components/OnDialog/index.vue";
import type { FormField } from "@/components/OnForm/types";
import type { TableColumn } from "@/components/OnTable/types";
import OnTable from "@/components/OnTable/index.vue";
import { useCrud, useMessage } from "@/hooks";

const { confirm, prompt, success, error } = useMessage();

const showCreate = ref(false);
const form = reactive({ username: "", password: "" });

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
  create,
} = useCrud({
  getListApi: (params) =>
    api.get("/api/rbac/users", { params }).then((r) => r.data),
  isPage: true,
  pageSize: 20,
});

// 搜索表单
const searchFields: FormField[] = [
  { prop: "username", label: "login.username", type: "input", span: 8 },
  {
    prop: "disabled",
    label: "common.status",
    type: "select",
    span: 8,
    options: [
      { label: "common.enabled", value: false },
      { label: "common.disabled", value: true },
    ],
  },
];

// 创建表单
const formFields: FormField[] = [
  { prop: "username", label: "login.username", type: "input", required: true },
  {
    prop: "password",
    label: "login.password",
    type: "password",
    required: true,
    showPassword: true,
  },
];

// 表格列
const tableColumns: TableColumn[] = [
  { prop: "id", label: "ID", width: 60 },
  { prop: "username", label: "login.username" },
  { prop: "roles", label: "rbac.colRoles" },
  { prop: "disabled", label: "common.status", width: 80, slot: "status" },
  {
    label: "common.action",
    width: 200,
    buttons: [
      { name: "rbac.resetPassword", command: "resetPwd", size: "small" },
      {
        name: "common.delete",
        command: "delete",
        type: "danger",
        size: "small",
      },
    ],
  },
];

function onPageChange(p: number) {
  page.value = p;
  load();
}

function handleCommand(command: string | number, row: any) {
  if (command === "resetPwd") resetPwd(row);
  else if (command === "delete") del(row);
}

onMounted(() => load());

async function submit() {
  if (!form.username || !form.password) return;
  const ok = await create(form);
  if (ok) {
    showCreate.value = false;
    form.username = "";
    form.password = "";
  }
}

async function resetPwd(row: any) {
  const value = await prompt({
    message: "rbac.resetPassword",
    inputValue: "123456",
  });
  if (!value) return;
  try {
    const { data } = await api.post(
      `/api/rbac/users/${row.id}/reset-password`,
      { new_password: value },
    );
    if (data.code === 0) success("common.success");
    else error(data.message);
  } catch {}
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
</script>

<style scoped>
.search-bar {
  display: flex;
  gap: 12px;
  align-items: center;
  margin-bottom: 12px;
}
</style>
