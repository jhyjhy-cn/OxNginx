<template>
  <div class="terminal-page">
    <!-- 终端区域 -->
    <div class="terminal-main">
      <div ref="terminalRef" class="terminal-container" />
      <div class="terminal-float">
        <el-select v-model="currentShell" size="small" @change="reconnect" style="width: 140px">
          <el-option value="powershell" label="PowerShell" />
          <el-option value="cmd" label="CMD" />
        </el-select>
      </div>
    </div>

    <!-- 常用命令侧栏 -->
    <div class="cmd-sidebar">
      <div class="cmd-header">
        <span>{{ t('terminal.quickCmd') }}</span>
        <el-button link size="small" @click="showAdd = true">
          <el-icon><Plus /></el-icon>
        </el-button>
      </div>
      <div class="cmd-list">
        <div v-for="(item, i) in commands" :key="i" class="cmd-item" @click="runCommand(item.cmd)">
          <div class="cmd-name">{{ item.name }}</div>
          <div class="cmd-cmd">{{ item.cmd }}</div>
          <el-button class="cmd-del" link size="small" @click.stop="removeCommand(i)">
            <el-icon><Close /></el-icon>
          </el-button>
        </div>
        <el-empty v-if="!commands.length" :description="t('terminal.noCmd')" :image-size="40" />
      </div>
    </div>

    <!-- 添加命令弹窗 -->
    <el-dialog v-model="showAdd" :title="t('terminal.addCmd')" width="400px" append-to-body>
      <el-form :model="newCmd" label-width="80px">
        <el-form-item :label="t('terminal.cmdName')">
          <el-input v-model="newCmd.name" :placeholder="t('terminal.cmdNamePh')" />
        </el-form-item>
        <el-form-item :label="t('terminal.cmdContent')">
          <el-input v-model="newCmd.cmd" :placeholder="t('terminal.cmdContentPh')" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showAdd = false">{{ t('common.cancel') }}</el-button>
        <el-button type="primary" @click="addCommand">{{ t('common.confirm') }}</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue'
import { useI18n } from 'vue-i18n'
import { Terminal } from '@xterm/xterm'
import { FitAddon } from '@xterm/addon-fit'
import { WebLinksAddon } from '@xterm/addon-web-links'
import '@xterm/xterm/css/xterm.css'
import { useAuthStore } from '@/stores/auth'

const { t } = useI18n()
const authStore = useAuthStore()
const terminalRef = ref<HTMLElement>()
const currentShell = ref('powershell')

// 常用命令
interface CmdItem { name: string; cmd: string }
const STORAGE_KEY = 'ox-terminal-cmds'
const commands = ref<CmdItem[]>(JSON.parse(localStorage.getItem(STORAGE_KEY) || '[]'))
const showAdd = ref(false)
const newCmd = ref<CmdItem>({ name: '', cmd: '' })

function saveCommands() {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(commands.value))
}

function addCommand() {
  if (!newCmd.value.name.trim() || !newCmd.value.cmd.trim()) return
  commands.value.push({ ...newCmd.value })
  saveCommands()
  newCmd.value = { name: '', cmd: '' }
  showAdd.value = false
}

function removeCommand(index: number) {
  commands.value.splice(index, 1)
  saveCommands()
}

function runCommand(cmd: string) {
  if (ws?.readyState === WebSocket.OPEN) {
    ws.send(cmd + '\r')
    terminal?.focus()
  }
}

// 终端
let terminal: Terminal | null = null
let fitAddon: FitAddon | null = null
let ws: WebSocket | null = null
let resizeObserver: ResizeObserver | null = null

function sendResize() {
  if (ws?.readyState === WebSocket.OPEN && terminal) {
    ws.send('\x01' + JSON.stringify({ cols: terminal.cols, rows: terminal.rows }))
  }
}

function connect() {
  const protocol = location.protocol === 'https:' ? 'wss:' : 'ws:'
  const token = authStore.token
  const cols = terminal?.cols ?? 80
  const rows = terminal?.rows ?? 24
  const url = `${protocol}//${location.host}/api/terminal/ws?token=${token}&cols=${cols}&rows=${rows}&shell=${currentShell.value}`
  ws = new WebSocket(url)

  ws.onopen = () => { terminal?.focus() }
  ws.onmessage = (e) => {
    if (e.data instanceof Blob) {
      e.data.arrayBuffer().then((buf) => { terminal?.write(new Uint8Array(buf)) })
    } else {
      terminal?.write(e.data)
    }
  }
  ws.onclose = () => { terminal?.writeln('\r\n\x1b[31m' + t('terminal.disconnected') + '\x1b[0m') }
  ws.onerror = () => { terminal?.writeln('\r\n\x1b[31m' + t('terminal.error') + '\x1b[0m') }
}

function reconnect() {
  ws?.close()
  terminal?.clear()
  setTimeout(() => connect(), 100)
}

onMounted(async () => {
  if (!terminalRef.value) return
  await document.fonts.ready

  terminal = new Terminal({
    cursorBlink: true,
    fontSize: 14,
    fontFamily: '"Maple Mono NF CN", Consolas, "Courier New", monospace',
    theme: {
      background: '#1e1e1e',
      foreground: '#d4d4d4',
      cursor: '#d4d4d4',
      selectionBackground: '#264f78',
    },
  })

  fitAddon = new FitAddon()
  terminal.loadAddon(fitAddon)
  terminal.loadAddon(new WebLinksAddon())
  terminal.open(terminalRef.value)
  fitAddon.fit()

  terminal.onData((data) => {
    if (ws?.readyState === WebSocket.OPEN) ws.send(data)
  })

  resizeObserver = new ResizeObserver(() => {
    fitAddon?.fit()
    sendResize()
  })
  resizeObserver.observe(terminalRef.value)
  connect()
})

onBeforeUnmount(() => {
  resizeObserver?.disconnect()
  ws?.close()
  terminal?.dispose()
})
</script>

<style scoped>
.terminal-page {
  display: flex;
  height: 100%;
  min-height: 0;
  gap: 8px;
}

/* 让父容器支持 flex 子元素撑满高度 */
:global(.content-area),
:global(.main-content) {
  display: flex;
  flex-direction: column;
}

/* 承接 content-area 的 flex:1 */
.terminal-page {
  flex: 1;
  min-height: 0;
}

.terminal-main {
  flex: 1;
  min-width: 0;
  position: relative;
}

.terminal-container {
  width: 100%;
  height: 100%;
  padding: 8px;
  box-sizing: border-box;
  background: #1e1e1e;
  border-radius: 4px;
}

.terminal-float {
  position: absolute;
  top: 16px;
  right: 16px;
  z-index: 10;
}

/* 常用命令侧栏 */
.cmd-sidebar {
  width: 220px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  background: var(--el-bg-color);
  border-radius: 4px;
  border: 1px solid var(--el-border-color-lighter);
  overflow: hidden;
}

.cmd-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 12px;
  font-size: 13px;
  font-weight: 600;
  border-bottom: 1px solid var(--el-border-color-lighter);
}

.cmd-list {
  flex: 1;
  overflow-y: auto;
  padding: 4px;
}

.cmd-item {
  position: relative;
  padding: 8px 28px 8px 10px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
}

.cmd-item:hover {
  background: var(--el-fill-color-light);
}

.cmd-name {
  font-weight: 500;
  margin-bottom: 2px;
  color: var(--el-text-color-primary);
}

.cmd-cmd {
  color: var(--el-text-color-secondary);
  font-family: var(--font-mono);
  font-size: 11px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.cmd-del {
  position: absolute;
  top: 6px;
  right: 4px;
  opacity: 0;
  transition: opacity 0.15s;
}

.cmd-item:hover .cmd-del {
  opacity: 1;
}
</style>
