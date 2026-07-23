<template>
  <el-dropdown trigger="hover" @command="onCommand">
    <!-- 触发按钮:Nginx 版本 + 运行状态 -->
    <el-button class="nginx-status-btn">
      <span class="dot" :class="dotClass" />
      <span>Nginx {{ status.version || '' }}</span>
      <el-divider direction="vertical" />
      <span class="state" :class="dotClass">{{ stateText }}</span>
      <el-icon class="caret"><ArrowDown /></el-icon>
    </el-button>

    <template #dropdown>
      <el-dropdown-menu>
        <el-dropdown-item command="toggle" :disabled="busy || status.not_installed">
          <el-icon><VideoPause v-if="status.running" /><VideoPlay v-else /></el-icon>
          {{ status.running ? $t('sys.dashboard.stop') : $t('sys.dashboard.start') }}
        </el-dropdown-item>
        <el-dropdown-item command="restart" :disabled="busy || !status.running || status.not_installed">
          <el-icon><RefreshRight /></el-icon>
          {{ $t('sys.dashboard.restart') }}
        </el-dropdown-item>
        <el-dropdown-item command="reload" :disabled="busy || !status.running || status.not_installed">
          <el-icon><Refresh /></el-icon>
          {{ $t('sys.dashboard.reloadConfig') }}
        </el-dropdown-item>
      </el-dropdown-menu>
    </template>
  </el-dropdown>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { ElMessage } from 'element-plus'
import { ArrowDown, VideoPlay, VideoPause, RefreshRight, Refresh } from '@element-plus/icons-vue'
import { startNginx, stopNginx, restartNginx, reloadNginx } from '@/api/nginx'
import { useWsStore } from '@/stores/ws'
import type { NginxStatus } from '@/api/nginx-status/type'

const { t } = useI18n()
const wsStore = useWsStore()

const status = ref<NginxStatus>({ running: false })
const busy = ref(false)

const dotClass = computed(() => {
  if (status.value.not_installed) return 'unknown'
  return status.value.running ? 'running' : 'stopped'
})

const stateText = computed(() => {
  if (status.value.not_installed) return t('sys.dashboard.notInstalled')
  return status.value.running ? t('sys.dashboard.running') : t('sys.dashboard.stopped')
})

// 订阅 dashboard 推送:新订阅时后端立即推一次快照,之后 10s 心跳刷新,
// nginx 操作后后端 trigger_dashboard_push 立即推 —— 全程无需手动查询
let wsUnsubscribe: (() => void) | null = null
function connectWs() {
  wsUnsubscribe = wsStore.subscribe('dashboard', (frame) => {
    if (frame.cmd !== 'dashboard') return
    const msg = frame.payload as { nginx?: NginxStatus }
    if (msg.nginx) status.value = msg.nginx
  })
}

async function onCommand(cmd: string) {
  if (busy.value || status.value.not_installed) return
  busy.value = true
  try {
    if (cmd === 'toggle') {
      if (status.value.running) {
        await stopNginx()
        ElMessage.success(t('sys.dashboard.nginxStopped'))
      } else {
        await startNginx()
        ElMessage.success(t('sys.dashboard.nginxStarted'))
      }
    } else if (cmd === 'restart') {
      await restartNginx()
      ElMessage.success(t('sys.dashboard.nginxRestarted'))
    } else if (cmd === 'reload') {
      await reloadNginx()
      ElMessage.success(t('sys.dashboard.configReloaded'))
    }
    // 不手动刷新:后端操作后已 trigger_dashboard_push,订阅会自动更新 status
  } catch (e: any) {
    ElMessage.error(e?.message || t('sys.dashboard.operationFailed'))
  } finally {
    busy.value = false
  }
}

onMounted(connectWs)
onUnmounted(() => {
  wsUnsubscribe?.()
  wsUnsubscribe = null
})
</script>

<style scoped>
.nginx-status-btn {
  display: inline-flex;
  align-items: center;
  gap: 2px;
}
.dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  margin-right: 4px;
  background: var(--el-color-info);
}
.dot.running {
  background: var(--el-color-success);
  animation: breathe 1.6s ease-in-out infinite;
}
.dot.stopped {
  background: var(--el-color-danger);
}
.state.running {
  color: var(--el-color-success);
}
.state.stopped {
  color: var(--el-color-danger);
}
.state.unknown {
  color: var(--el-color-info);
}
.caret {
  margin-left: 2px;
  font-size: 12px;
  color: var(--el-text-color-secondary);
}
@keyframes breathe {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.35;
  }
}
@media (prefers-reduced-motion: reduce) {
  .dot.running {
    animation: none;
  }
}
</style>
