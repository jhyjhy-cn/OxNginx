<template>
  <div class="log-page h-full">
    <el-card class="h-full">
      <div class="search-bar">
        <OnFormGrid
          :model="searchForm"
          :fields="searchFields"
          style="flex: 1"
        />
        <el-button type="primary" @click="doSearch">{{
          $t("common.search")
        }}</el-button>
        <el-button @click="doReset">{{ $t("common.reset") }}</el-button>
        <el-button type="success" @click="doExport">{{
          $t("common.download")
        }}</el-button>
      </div>

      <OnTable
        :data="dataList"
        :columns="tableColumns"
        :loading="loading"
        :pagination="{ total, currentPage: page, pageSize }"
        :options="{ height: 'auto' }"
        @page-change="onPageChange"
        @reload="load"
      >
        <template #type="{ row }">
          <el-tag
            :type="row.type === LoginLogType.Login ? 'primary' : 'info'"
            size="small"
          >
            {{
              row.type === LoginLogType.Login
                ? $t("sys.log.login")
                : $t("sys.log.logout")
            }}
          </el-tag>
        </template>
        <template #status="{ row }">
          <el-tag
            :type="row.status === LogStatus.Success ? 'success' : 'danger'"
            size="small"
          >
            {{
              row.status === LogStatus.Success
                ? $t("sys.log.loginSuccess")
                : $t("sys.log.loginFailed")
            }}
          </el-tag>
        </template>
        <template #created="{ row }">{{ formatTime(row.created_at) }}</template>
      </OnTable>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from "vue";
import dayjs from "dayjs";
import utc from "dayjs/plugin/utc";
import type { TableColumn } from "@/components/OnTable/types";
import type { FormField } from "@/components/OnForm/types";
import OnTable from "@/components/OnTable/index.vue";
import OnFormGrid from "@/components/OnForm/OnFormGrid/index.vue";
import { useCrud } from "@/hooks";
import { listLoginLogs } from "@/api/logs";
import { LoginLogType, LogStatus } from "@/consts";

dayjs.extend(utc);

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
  getListApi: (params) => listLoginLogs(queryParams(params)),
  isPage: true,
  pageSize: 20,
  searchForm: {
    username: "",
    ip: "",
    status: "",
    dateRange: null,
    start_time: "",
    end_time: "",
  },
});

const searchFields: FormField[] = [
  { prop: "username", label: "login.username", type: "input", span: 5 },
  { prop: "ip", label: "sys.log.ip", type: "input", span: 4 },
  {
    prop: "status",
    label: "common.status",
    type: "select",
    span: 4,
    options: [
      { label: "common.success", value: "success" },
      { label: "common.failed", value: "failed" },
    ],
  },
  { prop: "dateRange", label: "common.createdAt", type: "daterange", span: 8 },
];

function queryParams(params: Record<string, unknown> = searchForm) {
  const { dateRange: _, ...query } = params;
  return query;
}

const tableColumns: TableColumn[] = [
  { prop: "username", label: "login.username" },
  { prop: "ip", label: "sys.log.ip" },
  { prop: "os", label: "sys.log.os", showOverflowTooltip: true },
  { prop: "browser", label: "sys.log.browser", showOverflowTooltip: true },
  { prop: "type", label: "sys.log.type", slot: "type" },
  { prop: "status", label: "common.status", slot: "status" },
  { prop: "created_at", label: "common.createdAt", slot: "created" },
];

function formatTime(t: string | null): string {
  if (!t) return "";
  return dayjs.utc(t).local().format("YYYY-MM-DD HH:mm:ss");
}

function syncDates() {
  const range = searchForm.dateRange;
  if (Array.isArray(range)) {
    searchForm.start_time = range[0] + " 00:00:00";
    searchForm.end_time = range[1] + " 23:59:59";
  } else {
    searchForm.start_time = "";
    searchForm.end_time = "";
  }
}

function doSearch() {
  syncDates();
  search();
}
function doReset() {
  reset();
}

function onPageChange(p: number) {
  page.value = p;
  load();
}

function doExport() {
  syncDates();
  const query = Object.entries(queryParams())
    .filter(([, v]) => v !== "" && v !== undefined && v !== null)
    .map(([k, v]) => `${k}=${encodeURIComponent(v as string)}`)
    .join("&");
  window.open(`/api/log/login/export?${query}`, "_blank");
}

onMounted(load);
</script>

<style scoped>
.search-bar {
  display: flex;
  gap: 12px;
  align-items: center;
  margin-bottom: 12px;
  flex-wrap: wrap;
}
</style>
