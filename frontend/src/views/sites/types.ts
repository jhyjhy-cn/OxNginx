export interface Site {
  id: number
  name: string
  server_name: string
  listen: string
  ssl: number
  certificate_path: string | null
  key_path: string | null
  proxy_pass: string | null
  root_path: string | null
  remark: string | null
  expire_time: string | null
  rewrite_rules: string | null
  redirect_rules: string | null
  hotlink_config: string | null
  log_access_path: string | null
  log_error_path: string | null
  status: string
  created_at?: string
  cert_expire_time?: string
  cert_expire_days?: number
  backup_count?: number
  traffic?: Record<string, number>
}

export interface DomainItem {
  domain: string
}

export interface RewriteRule {
  pattern: string
  replacement: string
  flag: string
}

export interface RedirectRule {
  enabled: boolean // 开启重定向
  keep_params: boolean // 保留URL参数
  redirect_type: string // 'type' (域名类型) 或 'path' (路径)
  redirect_method: number // 301 永久重定向 / 302 临时重定向
  domains: string[] // 重定向域名列表（多选）
  target_url: string // 目标URL
  status: string // 'enabled' 或 'disabled'
}

export interface HotlinkCfg {
  enabled: boolean
  domainsStr: string
  return_code: number
}

export interface BackupFile {
  filename: string
  path: string
  size: number
  created_at: string
}

export interface ReverseProxy {
  id: number
  site_id: number
  name: string
  proxy_dir: string
  target_url: string
  cache: number
  status: string
  created_at?: string
  updated_at?: string
}
