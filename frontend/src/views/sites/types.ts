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
  domain: string
  target: string
  redirect_type: number
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
