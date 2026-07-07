<template>
  <div ref="terminalRef" class="terminal-container" />
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
  const url = `${protocol}//${location.host}/api/terminal/ws?token=${token}&cols=${cols}&rows=${rows}`
  console.log('[Terminal] Connecting to:', url)
  ws = new WebSocket(url)

  ws.onopen = () => {
    console.log('[Terminal] WebSocket opened')
    terminal?.writeln('\x1b[32m' + t('terminal.connected') + '\x1b[0m')
  }

  ws.onmessage = (e) => {
    console.log('[Terminal] Received message, type:', typeof e.data, 'isBlob:', e.data instanceof Blob, 'size:', e.data.size ?? e.data.length)
    if (e.data instanceof Blob) {
      e.data.arrayBuffer().then((buf) => {
        terminal?.write(new Uint8Array(buf))
      })
    } else {
      terminal?.write(e.data)
    }
  }

  ws.onclose = (e) => {
    console.log('[Terminal] WebSocket closed, code:', e.code, 'reason:', e.reason)
    terminal?.writeln('\r\n\x1b[31m' + t('terminal.disconnected') + '\x1b[0m')
  }

  ws.onerror = (e) => {
    console.error('[Terminal] WebSocket error:', e)
    terminal?.writeln('\r\n\x1b[31m' + t('terminal.error') + '\x1b[0m')
  }
}

onMounted(() => {
  if (!terminalRef.value) return

  terminal = new Terminal({
    cursorBlink: true,
    fontSize: 14,
    fontFamily: '"Maple Mono NF CN", "Maple Mono", Consolas, "Courier New", monospace',
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

  // 用户输入 → WebSocket
  terminal.onData((data) => {
    if (ws?.readyState === WebSocket.OPEN) {
      ws.send(data)
    }
  })

  // 监听容器尺寸变化 → fit + 通知后端
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
.terminal-container {
  width: 100%;
  height: calc(100vh - 160px);
  padding: 8px;
  box-sizing: border-box;
  background: #1e1e1e;
  border-radius: 4px;
}
</style>
