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
        @command="handleCommand"
        @reload="load"
      >
        <template #module="{ row }">{{ moduleLabel(row.module) }}</template>
        <template #status="{ row }">
          <el-tag
            :type="row.status === LogStatus.Success ? 'success' : 'danger'"
            size="small"
          >
            {{ row.status === LogStatus.Success ? "成功" : "失败" }}
          </el-tag>
        </template>
        <template #created="{ row }">{{ formatTime(row.created_at) }}</template>
        <template #cost="{ row }">{{ durationMs(row) }}</template>
      </OnTable>
    </el-card>

    <!-- 详情弹窗 -->
    <OnDialog v-model="showDialog" title="操作详情" width="700px">
      <el-descriptions :column="1" border size="small">
        <el-descriptions-item label="操作模块">{{
          moduleLabel(detail?.module)
        }}</el-descriptions-item>
        <el-descriptions-item label="操作类型">{{
          detail?.action
        }}</el-descriptions-item>
        <el-descriptions-item label="请求方式">{{
          detail?.method
        }}</el-descriptions-item>
        <el-descriptions-item label="操作人员">{{
          detail?.username
        }}</el-descriptions-item>
        <el-descriptions-item label="操作地址">{{
          detail?.uri
        }}</el-descriptions-item>
        <el-descriptions-item label="操作状态">
          <el-tag
            :type="detail?.status === LogStatus.Success ? 'success' : 'danger'"
            size="small"
          >
            {{ detail?.status === LogStatus.Success ? "成功" : "失败" }}
          </el-tag>
        </el-descriptions-item>
        <el-descriptions-item label="操作日期">{{
          formatTime(detail?.created_at ?? null)
        }}</el-descriptions-item>
        <el-descriptions-item label="消耗时间">{{
          durationMs(detail)
        }}</el-descriptions-item>
        <el-descriptions-item v-if="detail?.trace_id" label="TraceID">
          <span style="font-family: monospace; font-size: 12px">{{
            detail.trace_id
          }}</span>
        </el-descriptions-item>
        <el-descriptions-item label="请求参数">
          <pre class="detail-pre">{{
            formatJson(detail?.request_body ?? null)
          }}</pre>
        </el-descriptions-item>
        <el-descriptions-item v-if="detail?.error_msg" label="错误信息">
          <span style="color: var(--el-color-danger)">{{
            detail.error_msg
          }}</span>
        </el-descriptions-item>
      </el-descriptions>
    </OnDialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import dayjs from "dayjs";
import utc from "dayjs/plugin/utc";
import type { TableColumn } from "@/components/OnTable/types";
import type { FormField } from "@/components/OnForm/types";
import OnTable from "@/components/OnTable/index.vue";
import OnFormGrid from "@/components/OnForm/OnFormGrid/index.vue";
import OnDialog from "@/components/OnDialog/index.vue";
import { useCrud } from "@/hooks";
import { listOperationLogs } from "@/api/logs";
import { LogStatus } from "@/consts";

dayjs.extend(utc);

// ponytail: module 英文 key → 中文显示。前端做 i18n 翻译
const MODULE_OPTIONS = [
  { value: "site", label: "站点管理" },
  { value: "rbac", label: "权限管理" },
  { value: "nginx", label: "Nginx" },
  { value: "file", label: "文件管理" },
  { value: "config", label: "配置管理" },
  { value: "access", label: "访问控制" },
  { value: "backup", label: "备份管理" },
  { value: "template", label: "模板管理" },
  { value: "upstream", label: "上游服务" },
  { value: "proxy", label: "反向代理" },
  { value: "system", label: "系统设置" },
];
const MODULE_MAP: Record<string, string> = Object.fromEntries(
  MODULE_OPTIONS.map((m) => [m.value, m.label]),
);
function moduleLabel(key: string | null | undefined): string {
  if (!key) return "";
  return MODULE_MAP[key] || key;
}

function queryParams(params: Record<string, unknown> = searchForm) {
  const { dateRange: _, ...query } = params;
  return query;
}

const searchFields: FormField[] = [
  { prop: "username", label: "login.username", type: "input", span: 4 },
  {
    prop: "module",
    label: "操作模块",
    type: "select",
    span: 4,
    options: MODULE_OPTIONS,
  },
  { prop: "trace_id", label: "TraceID", type: "input", span: 5 },
  {
    prop: "status",
    label: "common.status",
    type: "select",
    span: 4,
    options: [
      { label: "成功", value: "success" },
      { label: "失败", value: "failed" },
    ],
  },
  { prop: "dateRange", label: "操作日期", type: "daterange", span: 7 },
];

interface OpLog {
  id: number;
  trace_id: string | null;
  username: string;
  module: string | null;
  action: string;
  method: string | null;
  uri: string | null;
  ip: string | null;
  status: LogStatus;
  cost_ms: number | null;
  duration_ms: number | null;
  request_body: string | null;
  response_body: string | null;
  error_msg: string | null;
  created_at: string | null;
}

function durationMs(row: OpLog | null | undefined): string {
  if (!row) return "-";
  const ms = row.duration_ms ?? row.cost_ms;
  return ms != null ? ms + "ms" : "-";
}

const showDialog = ref(false);
const detail = ref<OpLog | null>(null);

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
  getListApi: (params) => listOperationLogs(queryParams(params)),
  isPage: true,
  pageSize: 20,
  searchForm: {
    username: "",
    status: "",
    module: "",
    trace_id: "",
    dateRange: null,
    start_time: "",
    end_time: "",
  },
});

const tableColumns: TableColumn[] = [
  {
    prop: "module",
    label: "操作模块",
    showOverflowTooltip: true,
    slot: "module",
  },
  { prop: "action", label: "操作类型", showOverflowTooltip: true },
  { prop: "method", label: "请求方式" },
  { prop: "username", label: "login.username" },
  { prop: "uri", label: "操作地址", showOverflowTooltip: true },
  { prop: "status", label: "操作状态", slot: "status", width: 100 },
  { prop: "created_at", label: "操作日期", slot: "created" },
  { prop: "cost", label: "耗时", slot: "cost" },
  {
    label: "详情",
    width: 100,
    fixed: "right",
    buttons: [
      { name: { zh: "查看", en: "View" }, command: "detail", size: "small" },
    ],
  },
];

function formatTime(t: string | null): string {
  if (!t) return "";
  return dayjs.utc(t).local().format("YYYY-MM-DD HH:mm:ss");
}

function formatJson(s: string | null): string {
  if (!s) return "-";
  try {
    return JSON.stringify(JSON.parse(s), null, 2);
  } catch {
    return s;
  }
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

function handleCommand(command: string | number, row: OpLog) {
  if (command === "detail") {
    detail.value = row;
    showDialog.value = true;
  }
}

function doExport() {
  syncDates();
  const query = Object.entries(queryParams())
    .filter(([, v]) => v !== "" && v !== undefined && v !== null)
    .map(([k, v]) => `${k}=${encodeURIComponent(v as string)}`)
    .join("&");
  window.open(`/api/log/operation/export?${query}`, "_blank");
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
.detail-pre {
  margin: 0;
  font-size: 12px;
  white-space: pre-wrap;
  word-break: break-all;
  max-height: 200px;
  overflow-y: auto;
}
</style>
