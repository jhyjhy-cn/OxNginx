<template>
  <div class="sites">
    <el-card>
      <template #header>
        <div class="card-header">
          <span>{{ $t("sites.title") }}</span>
          <div>
            <el-button-group
              v-if="selectedSites.length > 0"
              style="margin-right: 12px"
            >
              <el-button size="small" @click="batchEnable">
                {{ $t("sites.batchEnable") }} ({{ selectedSites.length }})
              </el-button>
              <el-button size="small" @click="batchDisable">
                {{ $t("sites.batchDisable") }} ({{ selectedSites.length }})
              </el-button>
              <el-button size="small" type="danger" @click="batchDelete">
                {{ $t("sites.batchDelete") }} ({{ selectedSites.length }})
              </el-button>
            </el-button-group>
            <el-button type="primary" @click="showAddDialog">
              <el-icon><Plus /></el-icon>
              {{ $t("sites.addSite") }}
            </el-button>
          </div>
        </div>
      </template>

      <el-table
        :data="sites"
        style="width: 100%"
        v-loading="loading"
        @selection-change="handleSelectionChange"
      >
        <el-table-column type="selection" width="55" />
        <!-- 网站名 -->
        <el-table-column prop="name" :label="$t('sites.siteName')" width="150">
          <template #default="{ row }">
            <el-button type="primary" link @click="editSite(row)">{{
              row.name
            }}</el-button>
          </template>
        </el-table-column>
        <!-- 状态（可排序） -->
        <el-table-column
          :label="$t('common.status')"
          width="100"
          sortable
          :sort-method="(a: Site, b: Site) => (a.status === 'enabled' ? 0 : 1) - (b.status === 'enabled' ? 0 : 1)"
        >
          <template #default="{ row }">
            <el-switch
              :model-value="row.status === 'enabled'"
              inline-prompt
              active-text="启"
              inactive-text="停"
              @change="(val: boolean) => toggleSite(row, val)"
            />
          </template>
        </el-table-column>
        <!-- 备份 -->
        <el-table-column :label="$t('sites.backup')" width="100">
          <template #default="{ row }">
            <el-button
              v-if="row.backup_count > 0"
              type="primary"
              link
              @click="openBackup(row)"
            >{{ $t('sites.hasBackup', { n: row.backup_count }) }}</el-button>
            <el-button
              v-else
              type="info"
              link
              @click="openBackup(row)"
            >{{ $t('sites.noBackup') }}</el-button>
          </template>
        </el-table-column>
        <!-- 根目录（点击跳转文件管理） -->
        <el-table-column
          :label="$t('sites.rootPath')"
          min-width="180"
          show-overflow-tooltip
        >
          <template #default="{ row }">
            <el-button
              v-if="row.root_path"
              type="primary"
              link
              @click="openFileManager(row.root_path)"
            >{{ row.root_path }}</el-button>
            <span v-else>{{ row.proxy_pass || "-" }}</span>
          </template>
        </el-table-column>
        <!-- 日流量（表头下拉切换） -->
        <el-table-column width="140">
          <template #header>
            <el-dropdown @command="(cmd: string) => trafficMetric = cmd as any" trigger="click">
              <span class="traffic-header">
                {{ $t(`sites.traffic.${trafficMetric}`) }}
                <el-icon><ArrowDown /></el-icon>
              </span>
              <template #dropdown>
                <el-dropdown-menu>
                  <el-dropdown-item command="ip" :class="{ active: trafficMetric === 'ip' }">{{ $t('sites.traffic.ip') }}</el-dropdown-item>
                  <el-dropdown-item command="pv" :class="{ active: trafficMetric === 'pv' }">{{ $t('sites.traffic.pv') }}</el-dropdown-item>
                  <el-dropdown-item command="request" :class="{ active: trafficMetric === 'request' }">{{ $t('sites.traffic.request') }}</el-dropdown-item>
                  <el-dropdown-item command="uv" :class="{ active: trafficMetric === 'uv' }">{{ $t('sites.traffic.uv') }}</el-dropdown-item>
                </el-dropdown-menu>
              </template>
            </el-dropdown>
          </template>
          <template #default="{ row }">
            <span>{{ row.traffic?.[trafficMetric] ?? '-' }}</span>
          </template>
        </el-table-column>
        <!-- 到期时间（可排序） -->
        <el-table-column width="150" sortable :sort-method="sortExpireTime">
          <template #header>
            <span>{{ $t('sites.expireTime') }}</span>
          </template>
          <template #default="{ row }">
            <el-tag v-if="!row.expire_time" size="small">{{ $t('sites.permanent') }}</el-tag>
            <span v-else>{{ row.expire_time }}</span>
          </template>
        </el-table-column>
        <!-- 备注 -->
        <el-table-column
          prop="remark"
          :label="$t('sites.remark')"
          width="120"
          show-overflow-tooltip
        >
          <template #default="{ row }">
            {{ row.remark || '-' }}
          </template>
        </el-table-column>
        <!-- SSL证书（可排序） -->
        <el-table-column
          :label="$t('sites.sslCert')"
          width="140"
          sortable
          :sort-method="sortCert"
        >
          <template #default="{ row }">
            <el-tag
              v-if="row.ssl === 1 && row.cert_expire_days != null"
              :type="
                row.cert_expire_days > 30
                  ? 'success'
                  : row.cert_expire_days > 7
                    ? 'warning'
                    : 'danger'
              "
              size="small"
            >
              {{ $t("sites.daysRemaining", { n: row.cert_expire_days }) }}
            </el-tag>
            <el-tag v-else-if="row.ssl === 1" type="success" size="small">{{
              $t("sites.deployed")
            }}</el-tag>
            <el-tag v-else type="info" size="small">{{
              $t("sites.notDeployed")
            }}</el-tag>
          </template>
        </el-table-column>
        <!-- 操作 -->
        <el-table-column :label="$t('common.action')" width="200" fixed="right">
          <template #default="{ row }">
            <el-button type="primary" link @click="editSite(row)">{{
              $t("common.edit")
            }}</el-button>
            <el-button
              type="primary"
              link
              @click="deploySSL(row)"
              :loading="row._sslLoading"
              >{{ $t("sites.sslDeploy") }}</el-button
            >
            <el-button type="danger" link @click="deleteSite(row)">{{
              $t("common.delete")
            }}</el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- 添加站点对话框 -->
    <OnDialog
      v-model="addDialogVisible"
      :title="$t('sites.addSite')"
      width="600px"
    >
      <el-form
        ref="addFormRef"
        :model="addForm"
        :rules="addRules"
        label-width="100px"
      >
        <el-form-item :label="$t('sites.name')" prop="name">
          <el-input
            v-model="addForm.name"
            :placeholder="$t('sites.enterSiteName')"
          />
        </el-form-item>
        <el-form-item :label="$t('sites.domain')" prop="server_name">
          <el-input
            v-model="addForm.server_name"
            type="textarea"
            :autosize="{ minRows: 3, maxRows: 8 }"
            :placeholder="domainPlaceholder"
            @input="onDomainsInput"
          />
        </el-form-item>
        <el-form-item :label="$t('sites.enableSsl')">
          <el-switch v-model="addForm.ssl" />
        </el-form-item>
        <template v-if="addForm.ssl">
          <el-form-item :label="$t('sites.certPath')">
            <el-input
              v-model="addForm.certificate_path"
              placeholder="/opt/oxnginx/ssl/fullchain.cer"
            />
          </el-form-item>
          <el-form-item :label="$t('sites.keyPath')">
            <el-input
              v-model="addForm.key_path"
              placeholder="/opt/oxnginx/ssl/private.key"
            />
          </el-form-item>
        </template>
        <el-form-item :label="$t('sites.proxyPass')">
          <el-input
            v-model="addForm.proxy_pass"
            placeholder="http://127.0.0.1:8080"
          />
        </el-form-item>
        <el-form-item :label="$t('sites.rootPath')">
          <el-input
            v-model="addForm.root_path"
            :placeholder="$t('sites.rootPathHint')"
          />
        </el-form-item>
        <el-form-item :label="$t('sites.remark')">
          <el-input
            v-model="addForm.remark"
            type="textarea"
            :autosize="{ minRows: 2, maxRows: 4 }"
            :placeholder="$t('sites.remarkHint')"
          />
        </el-form-item>
        <el-form-item :label="$t('sites.expireTime')">
          <el-date-picker
            v-model="addForm.expire_time"
            type="datetime"
            :placeholder="$t('sites.permanent')"
            format="YYYY-MM-DD HH:mm:ss"
            value-format="YYYY-MM-DD HH:mm:ss"
            clearable
            style="width: 100%"
          />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="addDialogVisible = false">{{
          $t("common.cancel")
        }}</el-button>
        <el-button
          type="primary"
          :loading="submitting"
          @click="submitAddForm"
          >{{ $t("common.confirm") }}</el-button
        >
      </template>
    </OnDialog>

    <!-- 编辑站点对话框 -->
    <OnDialog
      v-model="editDialogVisible"
      :title="`${$t('sites.editSite')}[${editSiteName}]`"
      width="900px"
    >
      <el-tabs v-model="editActiveTab" tab-position="left" class="edit-tabs">
        <!-- 1. 域名管理 -->
        <el-tab-pane :label="$t('sites.tabDomain')" name="domain">
          <el-form label-width="80px">
            <el-form-item :label="$t('sites.domain')">
              <div style="display: flex; gap: 8px; width: 100%">
                <el-input
                  v-model="editDomainInput"
                  type="textarea"
                  :autosize="{ minRows: 2, maxRows: 6 }"
                  :placeholder="$t('sites.domainInputHint')"
                  style="flex: 1"
                />
                <el-button type="primary" style="align-self: flex-end" @click="addDomains">{{ $t('sites.addDomain') }}</el-button>
              </div>
            </el-form-item>
          </el-form>
          <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px">
            <el-button v-if="editDomainSelected.length > 0" type="danger" size="small" @click="deleteSelectedDomains">
              {{ $t('common.delete') }} ({{ editDomainSelected.length }})
            </el-button>
            <span v-else />
            <span style="font-size: 12px; color: #909399">{{ $t('sites.domainCount', { n: editDomains.length }) }}</span>
          </div>
          <el-table :data="editDomainsDisplay" style="width: 100%" max-height="380" @selection-change="(val: DomainItem[]) => editDomainSelected = val">
            <el-table-column type="selection" width="45" />
            <el-table-column :label="$t('sites.domain')">
              <template #default="{ row }">
                <el-button type="primary" link @click="openDomain(row.domain)">{{ row.domain }}</el-button>
              </template>
            </el-table-column>
            <el-table-column :label="$t('common.action')" width="80">
              <template #default="{ row }">
                <el-button type="danger" link size="small" @click="deleteDomain(row.domain)">{{ $t('common.delete') }}</el-button>
              </template>
            </el-table-column>
          </el-table>
        </el-tab-pane>

        <!-- 2. 伪静态 -->
        <el-tab-pane :label="$t('sites.tabRewrite')" name="rewrite">
          <div v-for="(rule, i) in editRewriteRules" :key="i" class="rule-row">
            <el-input v-model="rule.pattern" placeholder="^/old/(.*)$" style="flex: 2" @change="debouncedSave" />
            <el-input v-model="rule.replacement" placeholder="/new/$1" style="flex: 2" @change="debouncedSave" />
            <el-select v-model="rule.flag" style="width: 110px" @change="debouncedSave">
              <el-option label="last" value="last" />
              <el-option label="break" value="break" />
              <el-option label="redirect" value="redirect" />
              <el-option label="permanent" value="permanent" />
            </el-select>
            <el-button type="danger" :icon="Delete" circle size="small" @click="editRewriteRules.splice(i, 1); debouncedSave()" />
          </div>
          <el-button type="primary" plain size="small" @click="editRewriteRules.push({ pattern: '', replacement: '', flag: 'last' })">
            <el-icon><Plus /></el-icon> {{ $t('sites.addRule') }}
          </el-button>
        </el-tab-pane>

        <!-- 3. 配置文件 -->
        <el-tab-pane :label="$t('sites.tabConfig')" name="config">
          <div style="display: flex; gap: 8px; margin-bottom: 8px">
            <el-button type="primary" size="small" :loading="configSaving" @click="saveSiteConfig">{{ $t('common.save') }}</el-button>
            <el-button size="small" @click="loadSiteConfig">{{ $t('common.refresh') }}</el-button>
          </div>
          <div ref="configEditorRef" class="config-editor-box" />
        </el-tab-pane>

        <!-- 4. SSL证书 -->
        <el-tab-pane :label="$t('sites.tabSsl')" name="ssl">
          <el-form label-width="80px">
            <el-form-item :label="$t('sites.enableSsl')">
              <el-switch v-model="editForm.ssl" @change="debouncedSave" />
            </el-form-item>
            <template v-if="editForm.ssl">
              <el-form-item :label="$t('sites.certPath')">
                <el-input v-model="editForm.certificate_path" placeholder="/opt/oxnginx/ssl/fullchain.cer" @change="debouncedSave" />
              </el-form-item>
              <el-form-item :label="$t('sites.keyPath')">
                <el-input v-model="editForm.key_path" placeholder="/opt/oxnginx/ssl/private.key" @change="debouncedSave" />
              </el-form-item>
            </template>
          </el-form>
        </el-tab-pane>

        <!-- 5. 反向代理 -->
        <el-tab-pane :label="$t('sites.tabProxy')" name="proxy">
          <el-form label-width="80px">
            <el-form-item :label="$t('sites.proxyPass')">
              <el-input v-model="editForm.proxy_pass" placeholder="http://127.0.0.1:8080" @change="debouncedSave" />
            </el-form-item>
            <el-form-item :label="$t('sites.rootPath')">
              <el-input v-model="editForm.root_path" placeholder="/opt/oxnginx/wwwroot" @change="debouncedSave" />
            </el-form-item>
          </el-form>
        </el-tab-pane>

        <!-- 6. 重定向 -->
        <el-tab-pane :label="$t('sites.tabRedirect')" name="redirect">
          <el-alert :title="$t('sites.redirectHint')" type="info" :closable="false" style="margin-bottom: 12px" />
          <div v-for="(rule, i) in editRedirectRules" :key="i" class="rule-row">
            <el-input v-model="rule.domain" :placeholder="$t('sites.redirectDomain')" style="flex: 2" @change="debouncedSave" />
            <el-input v-model="rule.target" :placeholder="$t('sites.redirectTarget')" style="flex: 2" @change="debouncedSave" />
            <el-select v-model="rule.redirect_type" style="width: 90px" @change="debouncedSave">
              <el-option label="301" :value="301" />
              <el-option label="302" :value="302" />
            </el-select>
            <el-button type="danger" :icon="Delete" circle size="small" @click="editRedirectRules.splice(i, 1); debouncedSave()" />
          </div>
          <el-button type="primary" plain size="small" @click="editRedirectRules.push({ domain: '', target: '', redirect_type: 301 })">
            <el-icon><Plus /></el-icon> {{ $t('sites.addRule') }}
          </el-button>
        </el-tab-pane>

        <!-- 7. 防盗链 -->
        <el-tab-pane :label="$t('sites.tabHotlink')" name="hotlink">
          <el-form label-width="100px">
            <el-form-item :label="$t('sites.hotlinkEnable')">
              <el-switch v-model="editHotlink.enabled" @change="debouncedSave" />
            </el-form-item>
            <template v-if="editHotlink.enabled">
              <el-form-item :label="$t('sites.hotlinkDomains')">
                <el-input v-model="editHotlink.domainsStr" type="textarea" :autosize="{ minRows: 3, maxRows: 6 }" :placeholder="$t('sites.hotlinkDomainsHint')" @change="debouncedSave" />
              </el-form-item>
              <el-form-item :label="$t('sites.hotlinkCode')">
                <el-select v-model="editHotlink.return_code" @change="debouncedSave">
                  <el-option label="403 Forbidden" :value="403" />
                  <el-option label="404 Not Found" :value="404" />
                </el-select>
              </el-form-item>
            </template>
          </el-form>
        </el-tab-pane>

        <!-- 8. 网站日志 -->
        <el-tab-pane :label="$t('sites.tabLog')" name="log">
          <el-form label-width="100px">
            <el-form-item :label="$t('sites.logAccessPath')">
              <el-input v-model="editForm.log_access_path" :placeholder="$t('sites.logAccessPathHint')" @change="debouncedSave" />
            </el-form-item>
            <el-form-item :label="$t('sites.logErrorPath')">
              <el-input v-model="editForm.log_error_path" :placeholder="$t('sites.logErrorPathHint')" @change="debouncedSave" />
            </el-form-item>
          </el-form>
          <el-divider />
          <div style="display: flex; gap: 8px; margin-bottom: 8px">
            <el-button size="small" :loading="logLoading" @click="loadSiteLog('access')">{{ $t('sites.accessLog') }}</el-button>
            <el-button size="small" :loading="logLoading" @click="loadSiteLog('error')">{{ $t('sites.errorLog') }}</el-button>
          </div>
          <pre v-if="siteLog" class="log-output">{{ siteLog }}</pre>
          <el-empty v-else :description="$t('sites.clickToLoadLog')" />
        </el-tab-pane>
      </el-tabs>
    </OnDialog>

    <!-- 删除确认对话框 -->
    <OnDialog
      v-model="deleteDialogVisible"
      :title="$t('sites.deleteSite')"
      width="420px"
      :maximizable="false"
    >
      <div style="margin-bottom: 16px">
        <p>
          {{ $t("sites.confirmDeleteSite") }}
          <strong>{{ deleteTarget?.name }}</strong> ?
        </p>
      </div>
      <el-checkbox v-model="deleteOptions.deleteRecord">
        {{ $t("sites.deleteSiteRecord") }}
      </el-checkbox>
      <el-checkbox v-model="deleteOptions.deleteFiles" style="margin-top: 12px">
        {{
          $t("sites.deleteSiteFiles", {
            path: deleteTarget?.root_path || $t("common.none"),
          })
        }}
      </el-checkbox>
      <template #footer>
        <el-button @click="deleteDialogVisible = false">{{
          $t("common.cancel")
        }}</el-button>
        <el-button type="danger" @click="confirmDelete">{{
          $t("sites.confirmDelete")
        }}</el-button>
      </template>
    </OnDialog>

    <!-- 站点备份弹窗 -->
    <OnDialog
      v-model="backupDialogVisible"
      :title="$t('sites.backupDialogTitle', { name: backupSite?.name || '' })"
      width="800px"
    >
      <div style="margin-bottom: 12px; display: flex; justify-content: space-between; align-items: center">
        <el-button
          v-if="backupSelected.length > 0"
          type="danger"
          size="small"
          @click="batchDeleteBackups"
        >{{ $t('common.delete') }} ({{ backupSelected.length }})</el-button>
        <span v-else />
        <el-button type="primary" size="small" :loading="backupCreating" @click="createSiteBackup">
          <el-icon><Plus /></el-icon>
          {{ $t('sites.backupSite') }}
        </el-button>
      </div>
      <el-table
        :data="backupList"
        v-loading="backupLoading"
        style="width: 100%; height: 400px"
        height="400"
        @selection-change="(val: BackupFile[]) => backupSelected = val"
      >
        <el-table-column type="selection" width="45" />
        <el-table-column prop="filename" :label="$t('sites.backupFilename')" min-width="180" show-overflow-tooltip />
        <el-table-column :label="$t('sites.backupPath')" min-width="160" show-overflow-tooltip>
          <template #default="{ row }">
            <el-button type="primary" link @click="openFileManager(row.path)">{{ row.path }}</el-button>
          </template>
        </el-table-column>
        <el-table-column :label="$t('sites.backupSize')" width="90">
          <template #default="{ row }">
            {{ formatSize(row.size) }}
          </template>
        </el-table-column>
        <el-table-column prop="created_at" :label="$t('sites.backupTime')" width="160" />
        <el-table-column prop="remark" :label="$t('sites.remark')" width="100" show-overflow-tooltip>
          <template #default="{ row }">
            {{ row.remark || '-' }}
          </template>
        </el-table-column>
        <el-table-column :label="$t('common.action')" width="120" fixed="right">
          <template #default="{ row }">
            <el-button type="primary" link @click="downloadSiteBackup(row.filename)">{{ $t('common.download') }}</el-button>
            <el-button type="danger" link @click="deleteSiteBackup(row.filename)">{{ $t('common.delete') }}</el-button>
          </template>
        </el-table-column>
      </el-table>
      <div style="margin-top: 12px; display: flex; justify-content: flex-end">
        <el-pagination
          v-model:current-page="backupPage"
          v-model:page-size="backupPageSize"
          :total="backupTotal"
          :page-sizes="[10, 20, 50]"
          layout="total, sizes, prev, pager, next"
          small
          @current-change="onBackupPageChange"
          @size-change="onBackupPageChange"
        />
      </div>
    </OnDialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, nextTick, watch } from "vue";
import { useRouter } from "vue-router";
import { useI18n } from "vue-i18n";
import { ElMessage, ElMessageBox } from "element-plus";
import { Delete } from "@element-plus/icons-vue";
import type { FormInstance } from "element-plus";
import api from "@/api";
import OnDialog from "@/components/OnDialog/index.vue";
import { useTabStore } from "@/stores/tabs";
import { useFilesStore } from "@/stores/files";
import { monaco } from "@/utils/monaco-env";

const { t } = useI18n();
const router = useRouter();

interface Site {
  id: number;
  name: string;
  server_name: string;
  listen: string;
  ssl: number;
  certificate_path: string | null;
  key_path: string | null;
  proxy_pass: string | null;
  root_path: string | null;
  remark: string | null;
  expire_time: string | null;
  rewrite_rules: string | null;
  redirect_rules: string | null;
  hotlink_config: string | null;
  log_access_path: string | null;
  log_error_path: string | null;
  status: string;
  created_at?: string;
  cert_expire_time?: string;
  cert_expire_days?: number;
  backup_count?: number;
  traffic?: Record<string, number>;
}

const sites = ref<Site[]>([]);
const selectedSites = ref<Site[]>([]);
const loading = ref(false);
const submitting = ref(false);
const trafficMetric = ref<'ip' | 'pv' | 'request' | 'uv'>('ip');

// 添加站点
const addDialogVisible = ref(false);
const addFormRef = ref<FormInstance>();
const addForm = reactive({
  name: "",
  server_name: "",
  ssl: false,
  certificate_path: "",
  key_path: "",
  proxy_pass: "",
  root_path: "",
  remark: "",
  expire_time: "",
});
const addRules = {
  name: [
    { required: true, message: t("sites.enterSiteName"), trigger: "blur" },
  ],
  server_name: [
    { required: true, message: t("sites.enterDomain"), trigger: "blur" },
  ],
};

const domainPlaceholder = computed(() =>
  `${t('sites.domainHint')}\n${t('sites.domainFormatIp')}\n${t('sites.domainFormatPort')}\n${t('sites.domainFormatIpv6')}`
);

// 编辑站点
const editDialogVisible = ref(false);
const editId = ref<number | null>(null);
const editSiteName = ref("");
const editCreatedAt = ref("");
const editActiveTab = ref("domain");

interface DomainItem { domain: string }
const editDomainInput = ref('');
const editDomains = ref<string[]>([]);
const editDomainSelected = ref<DomainItem[]>([]);
const editDomainsDisplay = computed(() => editDomains.value.map(d => ({ domain: d })));

function openDomain(domain: string) {
  window.open('http://' + domain, '_blank');
}

function addDomains() {
  const lines = editDomainInput.value.split('\n').map(l => l.trim()).filter(Boolean);
  let added = false;
  for (const d of lines) {
    if (!editDomains.value.includes(d)) {
      editDomains.value.push(d);
      added = true;
    }
  }
  editDomainInput.value = '';
  if (added) saveDomains();
}

function deleteDomain(domain: string) {
  editDomains.value = editDomains.value.filter(d => d !== domain);
  saveDomains();
}

function deleteSelectedDomains() {
  const toDelete = new Set(editDomainSelected.value.map(d => d.domain));
  editDomains.value = editDomains.value.filter(d => !toDelete.has(d));
  editDomainSelected.value = [];
  saveDomains();
}

async function saveDomains() {
  if (!editId.value) return;
  try {
    const server_name = editDomains.value.join(' ');
    const listen = extractPort(editDomains.value[0] || '80');
    await api.put(`/api/sites/${editId.value}`, { server_name, listen });
    ElMessage.success(t("common.success"));
    fetchSites();
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || t("sites.operationFailed"));
  }
}

interface RewriteRule { pattern: string; replacement: string; flag: string }
interface RedirectRule { domain: string; target: string; redirect_type: number }
interface HotlinkCfg { enabled: boolean; domainsStr: string; return_code: number }

const editRewriteRules = ref<RewriteRule[]>([]);
const editRedirectRules = ref<RedirectRule[]>([]);
const editHotlink = reactive<HotlinkCfg>({ enabled: false, domainsStr: '', return_code: 403 });

// 配置文件编辑器
const configEditorRef = ref<HTMLElement>();
const configSaving = ref(false);
let configEditor: monaco.editor.IStandaloneCodeEditor | null = null;

// 网站日志
const logLoading = ref(false);
const siteLog = ref('');
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

onMounted(() => {
  fetchSites();
});

async function fetchSites() {
  loading.value = true;
  try {
    const response = await api.get("/api/sites/with-certs");
    if (response.data.code === 0) {
      sites.value = (response.data.data || []).map((s: Site) => {
        if (s.cert_expire_time) {
          const expireDate = new Date(s.cert_expire_time);
          const now = new Date();
          s.cert_expire_days = Math.ceil(
            (expireDate.getTime() - now.getTime()) / (1000 * 60 * 60 * 24),
          );
        }
        return s;
      });
    }
  } catch (error) {
    console.error("获取站点列表失败:", error);
  } finally {
    loading.value = false;
  }
}

function showAddDialog() {
  addForm.name = "";
  addForm.server_name = "";
  addForm.ssl = false;
  addForm.certificate_path = "";
  addForm.key_path = "";
  addForm.proxy_pass = "";
  addForm.root_path = "";
  addForm.remark = "";
  addForm.expire_time = "";
  addDialogVisible.value = true;
}

function onDomainsInput() {
  // 自动用第一个域名作为站点名（仅当站点名为空时）
  if (!addForm.name) {
    const firstLine = addForm.server_name.split("\n")[0]?.trim();
    if (firstLine) {
      // 去掉端口部分作为名称
      addForm.name = firstLine
        .replace(/:\d+\]?$|:\d+$/, "")
        .replace(/^\[|]$/g, "");
    }
  }
}

function openFileManager(path: string) {
  const tabStore = useTabStore();
  const filesStore = useFilesStore();
  tabStore.addTab({ path: '/files', title: 'menu.files', closable: true });
  // 归一化路径后比较，避免斜杠/大小写差异导致重复创建
  const normalized = path.replace(/\\/g, '/').replace(/\/+$/, '');
  const existing = filesStore.tabs.find(
    t => t.path.replace(/\\/g, '/').replace(/\/+$/, '') === normalized
  );
  if (existing) {
    filesStore.setActiveTab(existing.id);
  } else {
    filesStore.addTab(path);
  }
  router.push('/files');
}

function sortExpireTime(a: Site, b: Site) {
  const va = a.expire_time ? new Date(a.expire_time).getTime() : Infinity;
  const vb = b.expire_time ? new Date(b.expire_time).getTime() : Infinity;
  return va - vb;
}

function sortCert(a: Site, b: Site) {
  const va = a.cert_expire_days ?? 9999;
  const vb = b.cert_expire_days ?? 9999;
  return va - vb;
}

function editSite(site: Site) {
  editId.value = site.id;
  editSiteName.value = site.name;
  editCreatedAt.value = site.created_at
    ? new Date(site.created_at).toLocaleString()
    : "-";
  editForm.name = site.name;
  editForm.server_name = site.server_name;
  editForm.ssl = !!site.ssl;
  // 解析域名列表
  editDomains.value = site.server_name.split(' ').map(d => d.trim()).filter(Boolean);
  editDomainInput.value = '';
  editDomainSelected.value = [];
  editForm.certificate_path = site.certificate_path || "";
  editForm.key_path = site.key_path || "";
  editForm.proxy_pass = site.proxy_pass || "";
  editForm.root_path = site.root_path || "";
  editForm.remark = site.remark || "";
  editForm.expire_time = site.expire_time || "";
  editForm.rewrite_rules = site.rewrite_rules || "";
  editForm.redirect_rules = site.redirect_rules || "";
  editForm.hotlink_config = site.hotlink_config || "";
  editForm.log_access_path = site.log_access_path || "";
  editForm.log_error_path = site.log_error_path || "";

  // 解析伪静态规则
  try { editRewriteRules.value = JSON.parse(editForm.rewrite_rules || '[]') } catch { editRewriteRules.value = [] }
  // 解析重定向规则
  try { editRedirectRules.value = JSON.parse(editForm.redirect_rules || '[]') } catch { editRedirectRules.value = [] }
  // 解析防盗链
  try {
    const hc = JSON.parse(editForm.hotlink_config || '{}')
    editHotlink.enabled = hc.enabled || false
    editHotlink.domainsStr = (hc.allowed_domains || []).join('\n')
    editHotlink.return_code = hc.return_code || 403
  } catch {
    editHotlink.enabled = false; editHotlink.domainsStr = ''; editHotlink.return_code = 403
  }

  editActiveTab.value = "domain";
  siteLog.value = '';
  editDialogVisible.value = true;
}

function extractPort(domains: string): string {
  const first = domains.split('\n')[0]?.trim() || ''
  // IPv6: [addr]:port
  const ipv6Match = first.match(/^\[.+?\]:(\d+)$/)
  if (ipv6Match) return ipv6Match[1]
  // domain:port or ip:port (but not IPv6)
  const portMatch = first.match(/:(\d+)$/)
  if (portMatch) return portMatch[1]
  return '80'
}

async function submitAddForm() {
  const valid = await addFormRef.value?.validate().catch(() => false);
  if (!valid) return;

  submitting.value = true;
  try {
    const data = {
      name: addForm.name,
      server_name: addForm.server_name.replace(/\n/g, " ").trim(),
      listen: extractPort(addForm.server_name),
      ssl: addForm.ssl,
      certificate_path: addForm.certificate_path || null,
      key_path: addForm.key_path || null,
      proxy_pass: addForm.proxy_pass || null,
      root_path: addForm.root_path || null,
      remark: addForm.remark || null,
      expire_time: addForm.expire_time || null,
    };
    await api.post("/api/sites", data);
    ElMessage.success(t("sites.createSuccess"));
    addDialogVisible.value = false;
    fetchSites();
  } catch (error: any) {
    ElMessage.error(
      error.response?.data?.message || t("sites.operationFailed"),
    );
  } finally {
    submitting.value = false;
  }
}

// 配置文件编辑器初始化（切换到 config tab 时）
watch(editActiveTab, (tab) => {
  if (tab === 'config' && configEditorRef.value && editId.value) {
    nextTick(() => {
      if (!configEditor) {
        configEditor = monaco.editor.create(configEditorRef.value!, {
          value: '',
          language: 'nginx',
          theme: 'vs-dark',
          minimap: { enabled: false },
          fontSize: 13,
          lineNumbers: 'on',
          scrollBeyondLastLine: false,
          automaticLayout: true,
          tabSize: 4,
        })
      }
      loadSiteConfig()
    })
  }
})

async function loadSiteConfig() {
  if (!editId.value) return
  try {
    const res = await api.get(`/api/config/file/${editSiteName.value}`)
    if (res.data.code === 0 && configEditor) {
      configEditor.setValue(res.data.data?.content || '')
    }
  } catch { /* ignore */ }
}

async function saveSiteConfig() {
  if (!editId.value || !configEditor) return
  configSaving.value = true
  try {
    const res = await api.put(`/api/config/file/${editSiteName.value}`, {
      content: configEditor.getValue(),
    })
    if (res.data.code === 0) {
      ElMessage.success(t("common.success"))
    } else {
      ElMessage.error(res.data.message)
    }
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || t("common.failed"))
  } finally {
    configSaving.value = false
  }
}

async function loadSiteLog(type: 'access' | 'error') {
  logLoading.value = true
  try {
    const res = await api.get(`/api/log/${type}`)
    if (res.data.code === 0) {
      siteLog.value = (res.data.data?.lines || []).join('\n')
    }
  } catch {
    siteLog.value = t('sites.logLoadFailed')
  } finally {
    logLoading.value = false
  }
}

async function saveAllSettings() {
  if (!editId.value) return;
  try {
    const data = {
      ssl: editForm.ssl,
      certificate_path: editForm.certificate_path || null,
      key_path: editForm.key_path || null,
      proxy_pass: editForm.proxy_pass || null,
      root_path: editForm.root_path || null,
      rewrite_rules: JSON.stringify(editRewriteRules.value),
      redirect_rules: JSON.stringify(editRedirectRules.value),
      hotlink_config: JSON.stringify({
        enabled: editHotlink.enabled,
        allowed_domains: editHotlink.domainsStr.split('\n').map(d => d.trim()).filter(Boolean),
        return_code: editHotlink.return_code,
      }),
      log_access_path: editForm.log_access_path || null,
      log_error_path: editForm.log_error_path || null,
    };
    await api.put(`/api/sites/${editId.value}`, data);
    ElMessage.success(t("common.success"));
    fetchSites();
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || t("sites.operationFailed"));
  }
}

// 防抖保存（字段变更时自动调用）
let saveTimer: ReturnType<typeof setTimeout> | null = null;
function debouncedSave() {
  if (saveTimer) clearTimeout(saveTimer);
  saveTimer = setTimeout(() => saveAllSettings(), 500);
}

async function toggleSite(site: Site, enable?: boolean) {
  const newStatus =
    enable !== undefined
      ? enable
        ? "enabled"
        : "disabled"
      : site.status === "enabled"
        ? "disabled"
        : "enabled";
  try {
    await api.put(`/api/sites/${site.id}`, { status: newStatus });
    ElMessage.success(
      newStatus === "enabled" ? t("common.enabled") : t("common.disabled"),
    );
    fetchSites();
  } catch (error: any) {
    ElMessage.error(
      error.response?.data?.message || t("sites.operationFailed"),
    );
  }
}

async function deploySSL(site: Site) {
  try {
    await ElMessageBox.confirm(
      t("sites.sslDeployConfirm", { domain: site.server_name }),
      t("sites.sslDeploy"),
      { type: "warning" },
    );
    const response = await api.post(`/api/sites/${site.id}/deploy-ssl`);
    if (response.data.code === 0) {
      ElMessage.success(t("sites.sslDeploySuccess"));
      fetchSites();
    } else {
      ElMessage.error(response.data.message || t("sites.deployFailed"));
    }
  } catch (error: any) {
    if (error !== "cancel") {
      ElMessage.error(
        error.response?.data?.message ||
          error.message ||
          t("sites.deployFailed"),
      );
    }
  }
}

const deleteDialogVisible = ref(false);
const deleteTarget = ref<Site | null>(null);
const deleteOptions = reactive({
  deleteRecord: true,
  deleteFiles: false,
});

function deleteSite(site: Site) {
  deleteTarget.value = site;
  deleteOptions.deleteRecord = true;
  deleteOptions.deleteFiles = false;
  deleteDialogVisible.value = true;
}

async function confirmDelete() {
  if (!deleteTarget.value) return;
  try {
    await api.delete(`/api/sites/${deleteTarget.value.id}`, {
      data: {
        delete_record: deleteOptions.deleteRecord,
        delete_files: deleteOptions.deleteFiles,
      },
    });
    ElMessage.success(t("sites.deleteSuccess"));
    deleteDialogVisible.value = false;
    fetchSites();
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || t("sites.deleteFailed"));
  }
}

// 站点备份弹窗
interface BackupFile {
  filename: string;
  path: string;
  size: number;
  created_at: string;
}

const backupDialogVisible = ref(false);
const backupSite = ref<Site | null>(null);
const backupList = ref<BackupFile[]>([]);
const backupLoading = ref(false);
const backupCreating = ref(false);
const backupSelected = ref<BackupFile[]>([]);
const backupPage = ref(1);
const backupPageSize = ref(20);
const backupTotal = ref(0);

function openBackup(site: Site) {
  backupSite.value = site;
  backupDialogVisible.value = true;
  backupPage.value = 1;
  fetchBackups();
}

function onBackupPageChange() {
  fetchBackups();
}

async function fetchBackups() {
  if (!backupSite.value) return;
  backupLoading.value = true;
  try {
    const res = await api.get(`/api/sites/${backupSite.value.id}/backups`, {
      params: { page: backupPage.value, page_size: backupPageSize.value },
    });
    if (res.data.code === 0) {
      backupList.value = res.data.data?.items || [];
      backupTotal.value = res.data.data?.total || 0;
    }
  } catch {
    backupList.value = [];
  } finally {
    backupLoading.value = false;
  }
}

async function createSiteBackup() {
  if (!backupSite.value) return;
  backupCreating.value = true;
  try {
    const res = await api.post(`/api/sites/${backupSite.value.id}/backups`);
    if (res.data.code === 0) {
      ElMessage.success(t("sites.backupCreated"));
      fetchBackups();
      fetchSites();
    } else {
      ElMessage.error(res.data.message);
    }
  } catch (error: any) {
    ElMessage.error(error.response?.data?.message || t("sites.backupCreateFailed"));
  } finally {
    backupCreating.value = false;
  }
}

async function downloadSiteBackup(filename: string) {
  if (!backupSite.value) return;
  const url = `/api/sites/${backupSite.value.id}/backups/${encodeURIComponent(filename)}/download`;
  const token = localStorage.getItem("token");
  const a = document.createElement("a");
  a.href = url + (token ? `?token=${token}` : "");
  a.download = filename;
  a.click();
}

async function deleteSiteBackup(filename: string) {
  if (!backupSite.value) return;
  try {
    await ElMessageBox.confirm(
      t("sites.confirmDeleteBackup", { name: filename }),
      t("common.tip"),
      { type: "warning" },
    );
    const res = await api.delete(`/api/sites/${backupSite.value.id}/backups/${encodeURIComponent(filename)}`);
    if (res.data.code === 0) {
      ElMessage.success(t("sites.backupDeleted"));
      fetchBackups();
      fetchSites();
    } else {
      ElMessage.error(res.data.message);
    }
  } catch (error: any) {
    if (error !== "cancel") {
      ElMessage.error(error.response?.data?.message || t("sites.backupDeleteFailed"));
    }
  }
}

async function batchDeleteBackups() {
  if (!backupSite.value || backupSelected.value.length === 0) return;
  try {
    await ElMessageBox.confirm(
      t("sites.confirmBatchDeleteBackup", { count: backupSelected.value.length }),
      t("common.warning"),
      { type: "warning" },
    );
    const res = await api.post(`/api/sites/${backupSite.value.id}/backups/batch-delete`, {
      filenames: backupSelected.value.map(b => b.filename),
    });
    if (res.data.code === 0) {
      ElMessage.success(t("sites.backupDeleted"));
      backupSelected.value = [];
      fetchBackups();
      fetchSites();
    } else {
      ElMessage.error(res.data.message);
    }
  } catch (error: any) {
    if (error !== "cancel") {
      ElMessage.error(error.response?.data?.message || t("sites.backupDeleteFailed"));
    }
  }
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return bytes + " B";
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + " KB";
  if (bytes < 1024 * 1024 * 1024) return (bytes / (1024 * 1024)).toFixed(1) + " MB";
  return (bytes / (1024 * 1024 * 1024)).toFixed(1) + " GB";
}

function handleSelectionChange(selection: Site[]) {
  selectedSites.value = selection;
}

async function batchEnable() {
  try {
    await ElMessageBox.confirm(
      t("sites.batchEnableConfirm", { count: selectedSites.value.length }),
      t("common.tip"),
    );
    const response = await api.post("/api/sites/batch/enable", {
      ids: selectedSites.value.map((s) => s.id),
    });
    if (response.data.code === 0) {
      ElMessage.success(
        t("sites.batchEnableSuccess", { count: response.data.data.success }),
      );
      fetchSites();
    }
  } catch (error: any) {
    if (error !== "cancel") {
      ElMessage.error(
        error.response?.data?.message || t("sites.operationFailed"),
      );
    }
  }
}

async function batchDisable() {
  try {
    await ElMessageBox.confirm(
      t("sites.batchDisableConfirm", { count: selectedSites.value.length }),
      t("common.tip"),
    );
    const response = await api.post("/api/sites/batch/disable", {
      ids: selectedSites.value.map((s) => s.id),
    });
    if (response.data.code === 0) {
      ElMessage.success(
        t("sites.batchDisableSuccess", { count: response.data.data.success }),
      );
      fetchSites();
    }
  } catch (error: any) {
    if (error !== "cancel") {
      ElMessage.error(
        error.response?.data?.message || t("sites.operationFailed"),
      );
    }
  }
}

async function batchDelete() {
  try {
    await ElMessageBox.confirm(
      t("sites.batchDeleteConfirm", { count: selectedSites.value.length }),
      t("common.warning"),
      {
        type: "warning",
      },
    );
    const response = await api.post("/api/sites/batch/delete", {
      ids: selectedSites.value.map((s) => s.id),
    });
    if (response.data.code === 0) {
      ElMessage.success(
        t("sites.batchDeleteSuccess", { count: response.data.data.success }),
      );
      fetchSites();
    }
  } catch (error: any) {
    if (error !== "cancel") {
      ElMessage.error(
        error.response?.data?.message || t("sites.operationFailed"),
      );
    }
  }
}
</script>

<style scoped>
.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}
.traffic-header {
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: 14px;
}
.traffic-header:hover {
  color: var(--el-color-primary);
}

/* 编辑弹窗左侧标签页 */
.edit-tabs {
  height: 500px;
}
.edit-tabs :deep(.el-tabs__header) {
  min-width: 120px;
}
.edit-tabs :deep(.el-tabs__content) {
  padding: 0 16px;
  overflow-y: auto;
}

/* 规则行 */
.rule-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

/* 配置文件编辑器 */
.config-editor-box {
  width: 100%;
  height: 420px;
  border: 1px solid var(--el-border-color-lighter);
  border-radius: 4px;
}

/* 日志输出 */
.log-output {
  background: #1e1e1e;
  color: #d4d4d4;
  padding: 12px;
  border-radius: 4px;
  font-size: 12px;
  font-family: 'Cascadia Code', 'Fira Code', monospace;
  max-height: 380px;
  overflow: auto;
  white-space: pre-wrap;
  word-break: break-all;
  margin: 0;
}
</style>

<style>
/* 非 scoped：下拉菜单 active 样式 */
.el-dropdown-menu__item.active {
  color: var(--el-color-primary);
  font-weight: 600;
}
</style>
