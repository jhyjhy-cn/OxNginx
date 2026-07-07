/**
 * 菜单配置 —— 所有布局和标签页共用
 * 修改此处即可同步侧边栏、顶部导航、标签页图标
 */

export interface MenuItem {
  path: string
  title: string   // i18n key
  icon: string     // Element Plus 图标组件名
}

export interface MenuGroup {
  index: string        // 唯一标识
  title: string        // i18n key
  icon: string
  children: MenuItem[]
}

// ========== 扁平菜单（左侧双栏、标签页） ==========
export const flatMenuItems: MenuItem[] = [
  { path: '/dashboard',  title: 'menu.dashboard',  icon: 'Odometer' },
  { path: '/sites',      title: 'menu.sites',       icon: 'Grid' },
  { path: '/ssl',        title: 'menu.ssl',         icon: 'Lock' },
  { path: '/upstreams',  title: 'menu.upstreams',   icon: 'Connection' },
  { path: '/access',     title: 'menu.access',      icon: 'Key' },
  { path: '/templates',  title: 'menu.templates',   icon: 'Files' },
  { path: '/logs',       title: 'menu.logs',        icon: 'Document' },
  { path: '/config',     title: 'menu.config',      icon: 'Edit' },
  { path: '/files',      title: 'menu.files',       icon: 'FolderOpened' },
  { path: '/terminal',  title: 'menu.terminal',    icon: 'Monitor' },
  { path: '/settings',   title: 'menu.settings',    icon: 'Setting' },
]

// ========== 分组菜单（左侧树形、顶部树形） ==========
export const groupedMenuItems: MenuGroup[] = [
  { index: 'site-group',     title: 'menu.sites',    icon: 'Grid',     children: [
    { path: '/sites',     title: 'menu.sites',     icon: 'Grid' },
    { path: '/upstreams', title: 'menu.upstreams', icon: 'Connection' },
    { path: '/templates', title: 'menu.templates', icon: 'Files' },
  ]},
  { index: 'security-group', title: 'menu.ssl',      icon: 'Lock',     children: [
    { path: '/ssl',    title: 'menu.ssl',    icon: 'Lock' },
    { path: '/access', title: 'menu.access', icon: 'Key' },
  ]},
  { index: 'config-group',   title: 'menu.config',   icon: 'Edit',     children: [
    { path: '/config', title: 'menu.config', icon: 'Edit' },
    { path: '/logs',   title: 'menu.logs',   icon: 'Document' },
    { path: '/files',  title: 'menu.files',  icon: 'FolderOpened' },
    { path: '/terminal', title: 'menu.terminal', icon: 'Monitor' },
  ]},
]

// ========== 路径 → 图标映射（标签页用） ==========
export const tabIconMap: Record<string, string> = Object.fromEntries(
  flatMenuItems.map(item => [item.path, item.icon]),
)
