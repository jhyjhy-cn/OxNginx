export interface NginxStatus {
  running: boolean
  pid?: number | null
  version?: string | null
  uptime?: string | null
  not_installed?: boolean
  [key: string]: unknown
}