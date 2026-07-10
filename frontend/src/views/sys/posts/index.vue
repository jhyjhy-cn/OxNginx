<template>
  <div class="posts-page h-full">
    <el-card class="h-full">
      <!-- 搜索栏 -->
      <div class="search-bar">
        <OnFormGrid :model="searchForm" :fields="searchFields" style="flex: 1" />
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
          <el-button type="primary" @click="openCreate">{{ $t("common.add") }}</el-button>
        </template>
        <template #status="{ row }">
          <el-tag size="small" :type="row.status === 'enabled' ? 'success' : 'info'">
            {{ row.status === "enabled" ? $t("common.enabled") : $t("common.disabled") }}
          </el-tag>
        </template>
      </OnTable>
    </el-card>

    <!-- 创建/编辑弹窗 -->
    <OnDialog v-model="showForm" :title="formTitle" width="450px">
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
import { ref, reactive, computed, onMounted } from "vue";
import OnForm from "@/components/OnForm/OnForm/index.vue";
import OnFormGrid from "@/components/OnForm/OnFormGrid/index.vue";
import OnDialog from "@/components/OnDialog/index.vue";
import type { FormField } from "@/components/OnForm/types";
import type { TableColumn } from "@/components/OnTable/types";
import OnTable from "@/components/OnTable/index.vue";
import { useCrud, useMessage } from "@/hooks";
import { listPosts, createPost, updatePost, deletePost } from "@/api/sys/posts";
import type { Post } from "@/api/sys/posts/type";

const { success, error, confirm } = useMessage();

const showForm = ref(false);
const isEdit = ref(false);
const editingId = ref<number | null>(null);
const formRef = ref<InstanceType<typeof OnForm>>();

const form = reactive({ code: "", name: "", sort: 0 });

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
  getListApi: listPosts,
  isPage: true,
  pageSize: 20,
});

const searchFields: FormField[] = [
  { prop: "keyword", label: "common.search", type: "input", span: 8 },
];

const tableColumns: TableColumn[] = [
  { prop: "code", label: "sys.rbac.colCode", width: 160 },
  { prop: "name", label: "sys.rbac.colName" },
  { prop: "sort", label: "sys.rbac.colSort", width: 100 },
  { prop: "status", label: "common.status", width: 100, slot: "status" },
  {
    label: "common.action",
    width: 160,
    buttons: [
      { name: "common.edit", command: "edit", size: "small" },
      { name: "common.delete", command: "delete", type: "danger", size: "small" },
    ],
  },
];

const formFields = computed<FormField[]>(() => [
  { prop: "code", label: "sys.rbac.colCode", type: "input", required: true, disabled: isEdit.value },
  { prop: "name", label: "sys.rbac.colName", type: "input", required: true },
  { prop: "sort", label: "sys.rbac.colSort", type: "number", min: 0, max: 9999 },
]);

const formTitle = computed(() => (isEdit.value ? "common.edit" : "common.add"));

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
  Object.assign(form, { code: "", name: "", sort: 0 });
  showForm.value = true;
}

function openEdit(row: Post) {
  isEdit.value = true;
  editingId.value = row.id;
  Object.assign(form, { code: row.code, name: row.name, sort: row.sort });
  showForm.value = true;
}

async function submit() {
  if (!formRef.value) return;
  try {
    await formRef.value.validate();
  } catch {
    return;
  }
  const payload = { code: form.code, name: form.name, sort: form.sort };
  try {
    if (isEdit.value) {
      await updatePost(editingId.value!, payload);
      success("common.success");
    } else {
      await createPost(payload);
      success("common.success");
    }
    showForm.value = false;
    load();
  } catch (e: any) {
    error(e?.message || "common.fail");
  }
}

async function del(row: Post) {
  const ok = await confirm({ message: "common.confirmDelete" });
  if (!ok) return;
  try {
    await deletePost(row.id);
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
