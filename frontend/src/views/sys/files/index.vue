<template>
  <div class="files-page h-full">
    <el-card class="h-full">
      <div class="flex h-full" style="flex-direction: column">
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
        @selectionChange="(rows: any[]) => (selectedRows = rows)"
      >
        <template #toolbar-left>
          <el-upload
            :action="uploadUrl"
            :headers="uploadHeaders"
            :show-file-list="false"
            :before-upload="beforeUpload"
            :on-success="onUploadSuccess"
            :on-error="onUploadError"
            name="file"
            accept="*/*"
          >
            <el-button type="primary" :loading="uploading">
              {{ $t("common.upload") }}
            </el-button>
          </el-upload>
          <el-button
            type="danger"
            :disabled="!selectedRows.length"
            @click="batchDelete"
          >
            {{ $t("common.batchDelete") }} ({{ selectedRows.length }})
          </el-button>
        </template>
        <template #preview="{ row }">
          <el-image
            v-if="isImage(row.suffix)"
            :src="row.url || row.path"
            :preview-src-list="[row.url || row.path]"
            :preview-teleported="true"
            fit="cover"
            style="width: 40px; height: 40px; border-radius: 4px; cursor: pointer"
            hide-on-click-modal
          />
          <el-tag v-else size="small">{{ $t("sys.files.otherType") }}</el-tag>
        </template>
        <template #size="{ row }">
          {{ formatSize(row.size) }}
        </template>
      </OnTable>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import api from "@/api";
import OnFormGrid from "@/components/OnForm/OnFormGrid/index.vue";
import type { FormField } from "@/components/OnForm/types";
import type { TableColumn } from "@/components/OnTable/types";
import OnTable from "@/components/OnTable/index.vue";
import { useCrud, useMessage } from "@/hooks";

const { success, error, confirm } = useMessage();

const selectedRows = ref<any[]>([]);
const uploading = ref(false);

const uploadUrl = "/api/rbac/files/upload";
const uploadHeaders = {
  Authorization: `Bearer ${localStorage.getItem("token") || ""}`,
};

const searchFields: FormField[] = [
  { prop: "keyword", label: "sys.files.originalName", type: "input", span: 8 },
  { prop: "suffix", label: "sys.files.suffix", type: "input", span: 8 },
  { prop: "provider", label: "sys.files.provider", type: "input", span: 8 },
];

const tableColumns: TableColumn[] = [
  { type: "selection", width: 48 },
  { prop: "preview", label: "sys.files.preview", width: 80, slot: "preview" },
  { prop: "original_name", label: "sys.files.originalName", showOverflowTooltip: true },
  { prop: "suffix", label: "sys.files.suffix", width: 100 },
  { prop: "size", label: "sys.files.size", width: 120, slot: "size" },
  { prop: "provider", label: "sys.files.provider", width: 100 },
  { prop: "created_at", label: "common.createdAt", width: 180 },
  {
    label: "common.action",
    width: 200,
    buttons: [
      { name: "sys.files.download", command: "download", size: "small" },
      { name: "common.delete", command: "delete", type: "danger", size: "small" },
    ],
  },
];

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
    api.get("/api/rbac/files/page", { params }).then((r) => r.data),
  isPage: true,
  pageSize: 20,
});

function onPageChange(p: number) {
  page.value = p;
  load();
}

function handleCommand(command: string | number, row: any) {
  if (command === "download") download(row);
  else if (command === "delete") del(row);
}

function beforeUpload(_file: File) {
  uploading.value = true;
  return true;
}

function onUploadSuccess(res: any) {
  uploading.value = false;
  if (res?.code === 0) {
    success("sys.files.uploadSuccess");
    load();
  } else {
    error(res?.message || "sys.files.uploadFailed");
  }
}

function onUploadError(_err: any) {
  uploading.value = false;
  error("sys.files.uploadFailed");
}

function download(row: any) {
  window.open(`/api/rbac/files/${row.id}/download?_t=${Date.now()}`, "_blank");
}

async function del(row: any) {
  const ok = await confirm({ message: "sys.files.deleteConfirm" });
  if (!ok) return;
  try {
    await api.delete(`/api/rbac/files/${row.id}`);
    success("common.success");
    selectedRows.value = selectedRows.value.filter((r) => r.id !== row.id);
    load();
  } catch (e: any) {
    error(e?.response?.data?.message || e?.message);
  }
}

async function batchDelete() {
  if (!selectedRows.value.length) return;
  const ok = await confirm({
    message: "sys.files.batchDeleteConfirm",
    params: { n: selectedRows.value.length },
  });
  if (!ok) return;
  try {
    const ids = selectedRows.value.map((r) => r.id);
    const { data } = await api.post("/api/rbac/files/batch-delete", { ids });
    if (data.code === 0) {
      success(data.data);
      selectedRows.value = [];
      load();
    } else {
      error(data.message);
    }
  } catch (e: any) {
    error(e?.response?.data?.message || e?.message);
  }
}

function isImage(suffix: string) {
  return ["jpg", "jpeg", "png", "gif", "webp", "bmp", "svg"].includes(
    (suffix || "").toLowerCase(),
  );
}

function formatSize(bytes: number): string {
  if (!bytes) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  let i = 0;
  let n = bytes;
  while (n >= 1024 && i < units.length - 1) {
    n /= 1024;
    i++;
  }
  return `${n.toFixed(n >= 10 || i === 0 ? 0 : 2)} ${units[i]}`;
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