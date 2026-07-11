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
          <el-button v-auth="'sys:param:add'" type="primary" @click="openCreate">
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
import OnForm from "@/components/OnForm/OnForm/index.vue";
import OnFormGrid from "@/components/OnForm/OnFormGrid/index.vue";
import OnDialog from "@/components/OnDialog/index.vue";
import type { FormField } from "@/components/OnForm/types";
import type { TableColumn } from "@/components/OnTable/types";
import OnTable from "@/components/OnTable/index.vue";
import { useCrud, useMessage } from "@/hooks";
import {
  pageParams,
  getParam,
  createParam,
  updateParam,
  deleteParam,
} from "@/api/sys/params";
import type { ParamPayload } from "@/api/sys/params/type";

const { success, error, confirm } = useMessage();

const showForm = ref(false);
const isEdit = ref(false);
const editingId = ref<number | null>(null);
const formRef = ref<InstanceType<typeof OnForm>>();

const form = reactive<ParamPayload>({
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
  getListApi: pageParams,
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
    const data = await getParam(row.id);
    Object.assign(form, {
      key: data.key || "",
      value: data.value || "",
      name: data.name || "",
      group_code: data.group_code || "default",
      remark: data.remark || "",
      sort: data.sort ?? 0,
    });
    showForm.value = true;
  } catch (e: any) {
    error(e?.message || "common.fail");
  }
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
    if (isEdit.value) {
      await updateParam(editingId.value!, payload);
      success("sys.params.updateSuccess");
    } else {
      await createParam(payload);
      success("sys.params.createSuccess");
    }
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
    await deleteParam(row.id);
    success("common.success");
    load();
  } catch (e: any) {
    error(e?.message || "common.fail");
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