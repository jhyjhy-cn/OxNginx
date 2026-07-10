<template>
  <div class="params-page h-full">
    <el-card class="h-full">
      <!-- 搜索栏 -->
      <div class="search-bar">
        <OnFormGrid
          :model="searchForm"
          :fields="searchFields"
          style="flex: 1"
        />
        <el-button type="primary" @click="search">{{ $t("common.search") }}</el-button>
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
      >
        <template #toolbar-left>
          <el-button type="primary" @click="openCreate">
            {{ $t("common.add") }}
          </el-button>
        </template>
      </OnTable>
    </el-card>

    <!-- 创建/编辑弹窗 -->
    <OnDialog v-model="showForm" :title="formTitle" width="600px">
      <OnForm ref="formRef" :model="form">
        <OnFormGrid :fields="formFields" :model="form" />
      </OnForm>
      <template #footer>
        <el-button @click="showForm = false">{{ $t("common.cancel") }}</el-button>
        <el-button type="primary" @click="submit">{{ $t("common.confirm") }}</el-button>
      </template>
    </OnDialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, computed } from "vue";
import api from "@/api";
import OnForm from "@/components/OnForm/OnForm/index.vue";
import OnFormGrid from "@/components/OnForm/OnFormGrid/index.vue";
import OnDialog from "@/components/OnDialog/index.vue";
import type { FormField } from "@/components/OnForm/types";
import type { TableColumn } from "@/components/OnTable/types";
import OnTable from "@/components/OnTable/index.vue";
import { useCrud, useMessage } from "@/hooks";

const { success, error, confirm } = useMessage();

const showForm = ref(false);
const isEdit = ref(false);
const editingId = ref<number | null>(null);
const formRef = ref<InstanceType<typeof OnForm>>();

const form = reactive({
  key: "",
  value: "",
  name: "",
  group_code: "default",
  remark: "",
  sort: 0,
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
    api.get("/api/rbac/params", { params }).then((r) => r.data),
  isPage: true,
  pageSize: 20,
});

// 搜索字段
const searchFields: FormField[] = [
  { prop: "keyword", label: "sys.params.key", type: "input", span: 8 },
  { prop: "group_code", label: "sys.params.groupCode", type: "input", span: 8 },
];

// 表格列
const tableColumns: TableColumn[] = [
  { prop: "key", label: "sys.params.key", width: 220 },
  { prop: "name", label: "sys.params.name", width: 180 },
  { prop: "group_code", label: "sys.params.groupCode", width: 120 },
  { prop: "value", label: "sys.params.value", showOverflowTooltip: true },
  { prop: "remark", label: "common.remark", showOverflowTooltip: true },
  {
    label: "common.action",
    width: 160,
    buttons: [
      { name: "common.edit", command: "edit", size: "small" },
      { name: "common.delete", command: "delete", type: "danger", size: "small" },
    ],
  },
];

// 表单字段
const formFields = computed<FormField[]>(() => {
  return [
    { prop: "key", label: "sys.params.key", type: "input", required: true, disabled: isEdit.value },
    { prop: "name", label: "sys.params.name", type: "input", required: true },
    { prop: "group_code", label: "sys.params.groupCode", type: "input" },
    { prop: "value", label: "sys.params.value", type: "textarea", rows: 4 },
    { prop: "remark", label: "common.remark", type: "textarea", rows: 2 },
    { prop: "sort", label: "common.remark", type: "number" }, // ponytail: sort 直接复用 remark 文案，YAGNI 单独加 key
  ];
});

const formTitle = computed(() =>
  isEdit.value ? "common.edit" : "common.add"
);

function onPageChange(p: number) {
  page.value = p;
  load();
}

function handleCommand(command: string | number, row: any) {
  if (command === "edit") openEdit(row);
  else if (command === "delete") del(row);
}

function openCreate() {
  isEdit.value = false;
  editingId.value = null;
  Object.assign(form, {
    key: "",
    value: "",
    name: "",
    group_code: "default",
    remark: "",
    sort: 0,
  });
  showForm.value = true;
}

async function openEdit(row: any) {
  isEdit.value = true;
  editingId.value = row.id;
  try {
    const { data } = await api.get(`/api/rbac/params/${row.id}`);
    if (data.code === 0) {
      Object.assign(form, {
        key: data.data.key || "",
        value: data.data.value || "",
        name: data.data.name || "",
        group_code: data.data.group_code || "default",
        remark: data.data.remark || "",
        sort: data.data.sort ?? 0,
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
    return;
  }
  const payload = { ...form };
  try {
    const { data } = isEdit.value
      ? await api.put(`/api/rbac/params/${editingId.value}`, payload)
      : await api.post("/api/rbac/params", payload);
    if (data.code === 0) {
      success(isEdit.value ? "sys.params.updateSuccess" : "sys.params.createSuccess");
      showForm.value = false;
      load();
    } else {
      error(data.message);
    }
  } catch (e: any) {
    error(e?.response?.data?.message || e?.message);
  }
}

async function del(row: any) {
  const ok = await confirm({ message: "common.confirmDelete" });
  if (!ok) return;
  try {
    await api.delete(`/api/rbac/params/${row.id}`);
    success("common.success");
    load();
  } catch (e: any) {
    error(e?.response?.data?.message || e?.message);
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