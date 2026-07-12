/** 字符串 cmd 协议常量（与后端 modules/websocket/protocol.rs::cmd 对齐） */
export const IMCmd = {
  // 客户端 → 服务端
  PING: 'ping',
  SUBSCRIBE: 'subscribe',
  UNSUBSCRIBE: 'unsubscribe',
  TERMINAL_IN: 'terminal.in',
  TERMINAL_RESIZE: 'terminal.resize',
  TERMINAL_INIT: 'terminal.init',
  // 服务端 → 客户端
  PONG: 'pong',
  DASHBOARD: 'dashboard',
  EVENT: 'event',
} as const

export type Channel = 'dashboard' | 'events'

/** 客户端入站帧 */
export type ClientFrame =
  | { cmd: 'ping' }
  | { cmd: 'subscribe'; channels: Channel[] }
  | { cmd: 'unsubscribe'; channels: Channel[] }
  | { cmd: 'terminal.in'; data: string }
  | { cmd: 'terminal.resize'; cols: number; rows: number }
  | { cmd: 'terminal.init'; cols?: number; rows?: number; shell?: string }

/** 服务端事件 payload */
export type ServerEvent =
  | { type: 'Kick'; payload: { token_id: number; reason?: number } }
  | { type: 'Notice'; payload: { message: string } }

/** 服务端出站帧 */
export type ServerFrame =
  | { cmd: 'dashboard'; payload: { nginx?: Record<string, unknown>; stats?: Record<string, number> } }
  | { cmd: 'event'; payload: ServerEvent }
  | { cmd: 'pong' }

export function encodeClient(frame: ClientFrame): string {
  return JSON.stringify(frame)
}

/** 文本帧解析；解析失败返 null */
export function decodeServer(text: string): ServerFrame | null {
  try {
    const obj = JSON.parse(text)
    if (obj && typeof obj.cmd === 'string') return obj as ServerFrame
  } catch {}
  return null
}