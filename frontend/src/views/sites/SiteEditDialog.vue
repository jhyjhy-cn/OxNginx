<template>
  <OnDialog
    v-model="dialogVisible"
    :title="`${$t('sites.editSite')}[${siteName}]`"
    width="60%"
    height="70%"
  >
    <el-tabs v-model="activeTab" tab-position="left" class="edit-tabs">
      <!-- 1. 域名管理 -->
      <el-tab-pane :label="$t('sites.tabDomain')" name="domain">
        <DomainTab
          :site-id="props.siteId"
          v-model:domains="domains"
          @saved="emit('saved')"
        />
      </el-tab-pane>

      <!-- 2. 伪静态 -->
      <el-tab-pane :label="$t('sites.tabRewrite')" name="rewrite">
        <RewriteTab
          v-if="props.siteId"
          :site-id="props.siteId"
          v-model:rewrite-rules="editForm.rewrite_rules"
          @saved="emit('saved')"
        />
      </el-tab-pane>

      <!-- 3. 配置文件 -->
      <el-tab-pane :label="$t('sites.tabConfig')" name="config">
        <ConfigEditorTab
          v-if="props.siteId"
          :site-id="props.siteId"
          :site-name="props.siteName"
          @saved="emit('saved')"
        />
      </el-tab-pane>

      <!-- 4. 反向代理 -->
      <el-tab-pane :label="$t('sites.tabProxy')" name="proxy">
        <ProxyTab
          v-if="props.siteId"
          ref="proxyTabRef"
          :site-id="props.siteId"
        />
      </el-tab-pane>

      <!-- 5. 重定向 -->
      <el-tab-pane :label="$t('sites.tabRedirect')" name="redirect">
        <RedirectTab
          v-if="props.siteId"
          ref="redirectTabRef"
          :site-id="props.siteId"
          :site-name="props.siteName"
          :domains="domains"
          v-model:redirect-rules-json="editForm.redirect_rules"
          @saved="debouncedSave"
        />
      </el-tab-pane>

      <!-- 6. 防盗链 -->
      <el-tab-pane :label="$t('sites.tabHotlink')" name="hotlink">
        <HotlinkTab :hotlink="hotlink" @save="debouncedSave" />
      </el-tab-pane>

      <!-- 7. SSL证书 -->
      <el-tab-pane :label="$t('sites.tabSsl')" name="ssl">
        <SslTab :edit-form="editForm" @save="debouncedSave" />
      </el-tab-pane>

      <!-- 8. 网站日志 -->
      <el-tab-pane :label="$t('sites.tabLog')" name="log">
        <LogTab :edit-form="editForm" @save="debouncedSave" />
      </el-tab-pane>
    </el-tabs>
  </OnDialog>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch } from "vue";
import { useI18n } from "vue-i18n";
import { ElMessage } from "element-plus";
import api from "@/api";
import OnDialog from "@/components/OnDialog/index.vue";
import DomainTab from "./edits/DomainTab.vue";
import RewriteTab from "./edits/RewriteTab.vue";
import ConfigEditorTab from "./edits/ConfigEditorTab.vue";
import ProxyTab from "./edits/ProxyTab.vue";
import RedirectTab from "./edits/RedirectTab.vue";
import HotlinkTab from "./edits/HotlinkTab.vue";
import SslTab from "./edits/SslTab.vue";
import LogTab from "./edits/LogTab.vue";
import type { HotlinkCfg } from "./types";

const { t } = useI18n();

const props = defineProps<{
  visible: boolean;
  siteId: number | null;
  siteName: string;
}>();

const emit = defineEmits<{
  "update:visible": [value: boolean];
  saved: [];
}>();

const dialogVisible = computed({
  get: () => props.visible,
  set: (v) => emit("update:visible", v),
});

// ---- 通用状态 ----
const activeTab = ref("domain");
const domains = ref<string[]>([]);
const editForm = reactive({
  name: "",
  server_name: "",
  ssl: false,
  certificate_path: "",
  key_path: "",
  proxy_pass: "",
  root_path: "",
  remark: "",
  expire_time: "",
  rewrite_rules: "",
  redirect_rules: "",
  hotlink_config: "",
  log_access_path: "",
  log_error_path: "",
});

const hotlink = reactive<HotlinkCfg>({
  enabled: false,
  domainsStr: "",
  return_code: 403,
});

// ---- 子组件引用 ----
const proxyTabRef = ref<InstanceType<typeof ProxyTab> | null>(null);
const redirectTabRef = ref<InstanceType<typeof RedirectTab> | null>(null);

// ---- 初始化：打开弹窗时加载数据 ----
watch(
  () => props.visible,
  async (val) => {
    if (!val || !props.siteId) return;
    activeTab.value = "domain";

    try {
      const res = await api.get(`/api/sites/${props.siteId}`);
      if (res.data.code !== 0) return;
      const site = res.data.data;
      Object.assign(editForm, {
        name: site.name,
        server_name: site.server_name,
        ssl: !!site.ssl,
        certificate_path: site.certificate_path || "",
        key_path: site.key_path || "",
        proxy_pass: site.proxy_pass || "",
        root_path: site.root_path || "",
        remark: site.remark || "",
        expire_time: site.expire_time || "",
        rewrite_rules: site.rewrite_rules || "",
        redirect_rules: site.redirect_rules || "",
        hotlink_config: site.hotlink_config || "",
        log_access_path: site.log_access_path || "",
        log_error_path: site.log_error_path || "",
      });
      domains.value = site.server_name
        .split(" ")
        .map((d: string) => d.trim())
        .filter(Boolean);

      // 解析重定向规则
      redirectTabRef.value?.fetchRedirects();

      // 解析防盗链配置
      try {
        const hc = JSON.parse(editForm.hotlink_config || "{}");
        hotlink.enabled = hc.enabled || false;
        hotlink.domainsStr = (hc.allowed_domains || []).join("\n");
        hotlink.return_code = hc.return_code || 403;
      } catch {
        hotlink.enabled = false;
        hotlink.domainsStr = "";
        hotlink.return_code = 403;
      }
    } catch {
      /* ignore */
    }
  },
);

// ---- tab 切换时触发子组件加载 ----
watch(activeTab, (tab) => {
  if (tab === "proxy") {
    proxyTabRef.value?.fetchProxies();
  }
  if (tab === "redirect") {
    redirectTabRef.value?.fetchRedirects();
  }
});

// ---- 防抖保存 ----
let saveTimer: ReturnType<typeof setTimeout> | null = null;
function debouncedSave() {
  if (saveTimer) clearTimeout(saveTimer);
  saveTimer = setTimeout(() => saveAllSettings(), 500);
}

async function saveAllSettings() {
  if (!props.siteId) return;
  try {
    const data = {
      ssl: editForm.ssl,
      certificate_path: editForm.certificate_path || null,
      key_path: editForm.key_path || null,
      proxy_pass: editForm.proxy_pass || null,
      root_path: editForm.root_path || null,
      rewrite_rules: editForm.rewrite_rules || null,
      redirect_rules: editForm.redirect_rules || null,
      hotlink_config: JSON.stringify({
        enabled: hotlink.enabled,
        allowed_domains: hotlink.domainsStr
          .split("\n")
          .map((d) => d.trim())
          .filter(Boolean),
        return_code: hotlink.return_code,
      }),
      log_access_path: editForm.log_access_path || null,
      log_error_path: editForm.log_error_path || null,
    };
    await api.put(`/api/sites/${props.siteId}`, data);
    ElMessage.success(t("common.success"));
    emit("saved");
  } catch (error: any) {
    ElMessage.error(
      error.response?.data?.message || t("sites.operationFailed"),
    );
  }
}
</script>

<style scoped>
.edit-tabs {
  height: 100%;
  display: flex;
}
.edit-tabs :deep(.el-tabs__header) {
  min-width: 120px;
}
.edit-tabs :deep(.el-tabs__content) {
  padding: 0 16px;
  overflow-y: auto;
  flex: 1;
  min-height: 0;
}
.edit-tabs :deep(.el-tab-pane) {
  height: 100%;
}
</style>
