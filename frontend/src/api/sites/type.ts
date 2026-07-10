export interface Site {
  id: number
  name: string
  server_name?: string
  listen?: string
  status?: string
  config_path?: string
  enabled?: number | string
  ssl?: boolean | number
  [key: string]: unknown
}

export interface SitePayload {
  [key: string]: unknown
}

export interface Backup {
  id: number | string
  filename?: string
  size?: number
  created_at?: string
  [key: string]: unknown
}

export interface Upstream {
  id: number
  name?: string
  servers?: string[]
  [key: string]: unknown
}

export interface Proxy {
  id: number
  site_id?: number
  name?: string
  proxy_dir?: string
  target_url?: string
  path?: string
  target?: string
  cache?: number
  status?: string
  [key: string]: unknown
}

export interface AccessRule {
  id: number
  [key: string]: unknown
}

export interface Template {
  id: number
  name: string
  [key: string]: unknown
}

export interface SiteConfigFile {
  name: string
  content: string
  enabled?: boolean | number
  [key: string]: unknown
}