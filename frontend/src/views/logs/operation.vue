<template>
  <div class="log-page h-full">
    <el-card class="h-full">
      <OnFormGrid :model="searchForm" :fields="searchFields">
        <template #append>
          <el-button type="primary" @click="doSearch">{{
            $t("common.search")
          }}</el-button>
          <el-button @click="doReset">{{ $t("common.reset") }}</el-button>
        </template>
      </OnFormGrid>

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
        <template #toolbar-left>
          <el-button v-auth="'sys:opLog:export'" type="success" @click="doExport">
            {{ $t("common.export") }}
          </el-button>
        </template>
      </OnTable>
    </el-card>

    <!-- 详情弹窗 -->
    <OnDialog
      v-model="showDialog"
      title="sys.log.operationDetail"
      width="700px"
    >
      <el-descriptions :column="1" border size="small">
        <el-descriptions-item :label="$t('sys.log.module')">{{
          moduleLabel(detail?.module)
        }}</el-descriptions-item>
        <el-descriptions-item :label="$t('sys.log.action')">{{
          detail?.action
        }}</el-descriptions-item>
        <el-descriptions-item :label="$t('sys.log.method')">{{
          detail?.method
        }}</el-descriptions-item>
        <el-descriptions-item :label="$t('sys.log.operator')">{{
          detail?.username
        }}</el-descriptions-item>
        <el-descriptions-item :label="$t('sys.log.uri')">{{
          detail?.uri
        }}</el-descriptions-item>
        <el-descriptions-item :label="$t('sys.log.status')">
          <el-tag
            :type="detail?.status === LogStatus.Success ? 'success' : 'danger'"
            size="small"
          >
            {{ detail?.status === LogStatus.Success ? "成功" : "失败" }}
          </el-tag>
        </el-descriptions-item>
        <el-descriptions-item :label="$t('sys.log.createdAt')">{{
          formatTime(detail?.created_at ?? null)
        }}</el-descriptions-item>
        <el-descriptions-item :label="$t('sys.log.duration')">{{
          durationMs(detail)
        }}</el-descriptions-item>
        <el-descriptions-item v-if="detail?.trace_id" label="TraceID">
          <span style="font-family: monospace; font-size: 12px">{{
            detail.trace_id
          }}</span>
        </el-descriptions-item>
        <el-descriptions-item :label="$t('sys.log.requestParams')">
          <pre class="detail-pre">{{
            formatJson(detail?.request_body ?? null)
          }}</pre>
        </el-descriptions-item>
        <el-descriptions-item
          v-if="detail?.error_msg"
          :label="$t('sys.log.errorMsg')"
        >
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
import { downloadXlsx } from "@/utils/export";
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
    label: "sys.log.module",
    type: "select",
    span: 4,
    options: MODULE_OPTIONS,
  },
  { prop: "trace_id", label: "TraceID", type: "input", span: 4 },
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
  { prop: "dateRange", label: "sys.log.createdAt", type: "daterange", span: 4 },
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
    label: "sys.log.module",
    showOverflowTooltip: true,
    slot: "module",
  },
  { prop: "action", label: "sys.log.action", showOverflowTooltip: true },
  { prop: "method", label: "sys.log.method" },
  { prop: "username", label: "login.username" },
  { prop: "uri", label: "sys.log.uri", showOverflowTooltip: true },
  { prop: "status", label: "sys.log.status", slot: "status", width: 100 },
  { prop: "created_at", label: "sys.log.createdAt", slot: "created" },
  { prop: "cost", label: "sys.log.duration", slot: "cost" },
  {
    label: "sys.log.detail",
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

async function doExport() {
  syncDates();
  const params: Record<string, any> = {
    page_size: 10000,
    ...queryParams(),
  };
  Object.keys(params).forEach((k) => {
    if (params[k] === null || params[k] === undefined || params[k] === "") {
      delete params[k];
    }
  });
  const ts = dayjs().format("YYYY-MM-DD");
  await downloadXlsx("/api/log/operation/export", params, `operation_logs-${ts}.xlsx`);
}

onMounted(load);
</script>

<style scoped>
.search-bar {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  margin-bottom: 12px;
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
