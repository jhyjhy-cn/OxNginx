import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import {
  decodeServer,
  encodeClient,
  IMCmd,
  type Channel,
  type ClientFrame,
  type ServerFrame,
} from '@/utils/ws-protocol'
import { useAuthStore } from '@/stores/auth'

type Listener = (frame: ServerFrame) => void

/** 文本通道（dashboard + events 共享）：单连接，cmd-driven + 订阅模型 */
type TextListener = Listener
type TerminalListener = (data: Uint8Array) => void

/**
 * 统一 WS Hub：
 *  - textSocket：dashboard + events 共享，发送 SUBSCRIBE/UNSUBSCRIBE 切换推送
 *  - terminalSocket：terminal 单独一条（独立端点 /api/ws?type=terminal，binary 帧）
 *  - 心跳 30s；断线指数退避重连；logout 时 close
 */
export const useWsStore = defineStore('ws', () => {
  const status = ref<'idle' | 'connecting' | 'open' | 'closed'>('idle')

  let textSocket: WebSocket | null = null
  let textHeartbeat: number | undefined
  let textReconnect: number | undefined
  let textAttempt = 0

  let termSocket: WebSocket | null = null
  let termHeartbeat: number | undefined
  let termReconnect: number | undefined
  let termAttempt = 0

  const textListeners = new Map<Channel, Set<TextListener>>()
  const termListeners = new Set<TerminalListener>()
  let eventSubUnsub: (() => void) | null = null

  function ensureKindSet(kind: Channel) {
    if (!textListeners.has(kind)) textListeners.set(kind, new Set())
    return textListeners.get(kind)!
  }

  function emitText(kind: Channel, frame: ServerFrame) {
    const set = textListeners.get(kind)
    if (!set) return
    for (const l of set) {
      try { l(frame) } catch (e) { console.error('[ws] listener error:', e) }
    }
  }

  function buildUrl(extra?: Record<string, string>) {
    const proto = location.protocol === 'https:' ? 'wss:' : 'ws:'
    const token = useAuthStore().token
    const params = new URLSearchParams({ token })
    if (extra) for (const [k, v] of Object.entries(extra)) params.set(k, v)
    return `${proto}//${location.host}/api/ws?${params.toString()}`
  }

  function buildTerminalUrl(extra?: Record<string, string>) {
    const proto = location.protocol === 'https:' ? 'wss:' : 'ws:'
    const token = useAuthStore().token
    const params = new URLSearchParams({ type: 'terminal', token })
    if (extra) for (const [k, v] of Object.entries(extra)) params.set(k, v)
    return `${proto}//${location.host}/api/ws?${params.toString()}`
  }

  /** 同步当前所有订阅者到 server */
  function syncSubscriptions() {
    if (textSocket?.readyState !== WebSocket.OPEN) {
      console.log('[ws] syncSubscriptions: socket not open', textSocket?.readyState)
      return
    }
    const channels: Channel[] = []
    for (const [k, set] of textListeners) if (set.size > 0) channels.push(k)
    if (channels.length === 0) {
      console.log('[ws] syncSubscriptions: no channels, unsubscribe all')
      textSocket.send(encodeClient({ cmd: IMCmd.UNSUBSCRIBE, channels: ['dashboard', 'events'] }))
    } else {
      console.log('[ws] syncSubscriptions: sending', channels)
      textSocket.send(encodeClient({ cmd: IMCmd.SUBSCRIBE, channels }))
    }
  }

  function connectText() {
    const auth = useAuthStore()
    if (!auth.token) return
    if (textSocket && (textSocket.readyState === WebSocket.OPEN || textSocket.readyState === WebSocket.CONNECTING)) {
      // 已连：只同步订阅状态
      syncSubscriptions()
      return
    }

    status.value = 'connecting'
    textSocket = new WebSocket(buildUrl())

    textSocket.onopen = () => {
      status.value = 'open'
      textAttempt = 0
      textHeartbeat = window.setInterval(() => {
        if (textSocket?.readyState === WebSocket.OPEN) {
          textSocket.send(encodeClient({ cmd: IMCmd.PING }))
        }
      }, 30_000)
      // 握手成功立即同步订阅
      syncSubscriptions()
    }

    textSocket.onmessage = (e) => {
      console.log('[ws] recv', e.data)
      if (typeof e.data !== 'string') return
      const frame = decodeServer(e.data)
      if (!frame) return
      if (frame.cmd === IMCmd.DASHBOARD) emitText('dashboard', frame)
      else if (frame.cmd === IMCmd.EVENT) emitText('events', frame)
    }

    textSocket.onerror = () => {}
    textSocket.onclose = () => {
      status.value = 'closed'
      if (textHeartbeat) { window.clearInterval(textHeartbeat); textHeartbeat = undefined }
      textSocket = null
      if (useAuthStore().token) {
        if (textReconnect) return
        const delay = Math.min(30_000, 1000 * 2 ** textAttempt)
        textAttempt++
        textReconnect = window.setTimeout(() => { textReconnect = undefined; connectText() }, delay)
      }
    }
  }

  function close() {
    if (textReconnect) { window.clearTimeout(textReconnect); textReconnect = undefined }
    if (textHeartbeat) { window.clearInterval(textHeartbeat); textHeartbeat = undefined }
    if (textSocket) { textSocket.onclose = null; textSocket.close(); textSocket = null }
    if (termReconnect) { window.clearTimeout(termReconnect); termReconnect = undefined }
    if (termHeartbeat) { window.clearInterval(termHeartbeat); termHeartbeat = undefined }
    if (termSocket) { termSocket.onclose = null; termSocket.close(); termSocket = null }
    if (eventSubUnsub) { eventSubUnsub(); eventSubUnsub = null }
    status.value = 'closed'
  }

  function subscribe(kind: Channel, listener: TextListener): () => void {
    const set = ensureKindSet(kind)
    set.add(listener)
    if (status.value === 'idle' || status.value === 'closed') connectText()
    else syncSubscriptions() // 已连：同步给 server
    return () => {
      set.delete(listener)
      // 没人订阅了则断开；否则同步
      let any = false
      for (const s of textListeners.values()) if (s.size > 0) { any = true; break }
      if (!any) close()
      else syncSubscriptions()
    }
  }

  function setEventListener(fn: TextListener) {
    if (eventSubUnsub) return
    eventSubUnsub = subscribe('events', fn)
  }

  function send(frame: ClientFrame) {
    if (frame.cmd === IMCmd.TERMINAL_IN || frame.cmd === IMCmd.TERMINAL_RESIZE || frame.cmd === IMCmd.TERMINAL_INIT) {
      if (termSocket?.readyState === WebSocket.OPEN) {
        termSocket.send(encodeClient(frame))
      }
      return
    }
    if (textSocket?.readyState === WebSocket.OPEN) {
      textSocket.send(encodeClient(frame))
    }
  }

  function subscribeTerminal(listener: TerminalListener, init?: { cols?: number; rows?: number; shell?: string }): () => void {
    termListeners.add(listener)
    if (!termSocket || termSocket.readyState === WebSocket.CLOSED) {
      const extra: Record<string, string> = {}
      if (init?.cols) extra.cols = String(init.cols)
      if (init?.rows) extra.rows = String(init.rows)
      if (init?.shell) extra.shell = init.shell
      termSocket = new WebSocket(buildTerminalUrl(extra))
      termSocket.binaryType = 'arraybuffer'

      termSocket.onopen = () => {
        termAttempt = 0
        termHeartbeat = window.setInterval(() => {
          if (termSocket?.readyState === WebSocket.OPEN) {
            termSocket.send(encodeClient({ cmd: IMCmd.PING }))
          }
        }, 30_000)
      }
      termSocket.onmessage = (e) => {
        if (e.data instanceof ArrayBuffer) {
          const arr = new Uint8Array(e.data)
          for (const l of termListeners) {
            try { l(arr) } catch (err) { console.error('[ws:term] listener error:', err) }
          }
        }
      }
      termSocket.onerror = () => {}
      termSocket.onclose = () => {
        if (termHeartbeat) { window.clearInterval(termHeartbeat); termHeartbeat = undefined }
        termSocket = null
        if (useAuthStore().token && termListeners.size > 0) {
          if (termReconnect) return
          const delay = Math.min(30_000, 1000 * 2 ** termAttempt)
          termAttempt++
          termReconnect = window.setTimeout(() => { termReconnect = undefined }, delay)
          // 重新订阅
          setTimeout(() => {
            const extra: Record<string, string> = {}
            if (init?.cols) extra.cols = String(init.cols)
            if (init?.rows) extra.rows = String(init.rows)
            if (init?.shell) extra.shell = init.shell
            subscribeTerminal(listener, init)
          }, delay)
        }
      }
    }
    return () => {
      termListeners.delete(listener)
      if (termListeners.size === 0 && termSocket) {
        termSocket.onclose = null
        termSocket.close()
        termSocket = null
      }
    }
  }

  const isOpen = computed(() => status.value === 'open')
  return { status, isOpen, connect: connectText, close, subscribe, subscribeTerminal, send, setEventListener }
})