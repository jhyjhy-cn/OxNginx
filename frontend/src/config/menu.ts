/**
 * 菜单配置 —— 所有布局和标签页共用
 * 修改此处即可同步侧边栏、顶部导航、标签页图标
 */

export interface MenuItem {
  path: string
  title: string // i18n key
  icon: string // Element Plus 图标组件名
}

export interface MenuGroup {
  index: string // 唯一标识
  title: string // i18n key
  icon: string
  children: MenuItem[]
}

// ========== 扁平菜单（左侧双栏、标签页） ==========
export const flatMenuItems: MenuItem[] = [
  { path: '/dashboard', title: 'sys.menu.dashboard', icon: 'Odometer' },
  { path: '/sites', title: 'sys.menu.sites', icon: 'Grid' },
  { path: '/ssl', title: 'sys.menu.ssl', icon: 'Lock' },
  { path: '/upstreams', title: 'sys.menu.upstreams', icon: 'Connection' },
  { path: '/access', title: 'sys.menu.access', icon: 'Key' },
  { path: '/templates', title: 'sys.menu.templates', icon: 'Files' },
  { path: '/logs', title: 'sys.menu.logs', icon: 'Document' },
  { path: '/config', title: 'sys.menu.config', icon: 'Edit' },
  { path: '/files', title: 'sys.menu.files', icon: 'FolderOpened' },
  { path: '/terminal', title: 'sys.menu.terminal', icon: 'Monitor' },
  { path: '/settings', title: 'sys.menu.settings', icon: 'Setting' },
]

// ========== 分组菜单（左侧树形、顶部树形） ==========
export const groupedMenuItems: MenuGroup[] = [
  {
    index: 'site-group',
    title: 'sys.menu.sites',
    icon: 'Grid',
    children: [
      { path: '/sites', title: 'sys.menu.sites', icon: 'Grid' },
      { path: '/upstreams', title: 'sys.menu.upstreams', icon: 'Connection' },
      { path: '/templates', title: 'sys.menu.templates', icon: 'Files' },
    ],
  },
  {
    index: 'security-group',
    title: 'sys.menu.ssl',
    icon: 'Lock',
    children: [
      { path: '/ssl', title: 'sys.menu.ssl', icon: 'Lock' },
      { path: '/access', title: 'sys.menu.access', icon: 'Key' },
    ],
  },
  {
    index: 'config-group',
    title: 'sys.menu.config',
    icon: 'Edit',
    children: [
      { path: '/config', title: 'sys.menu.config', icon: 'Edit' },
      { path: '/logs', title: 'sys.menu.logs', icon: 'Document' },
      { path: '/files', title: 'sys.menu.files', icon: 'FolderOpened' },
      { path: '/terminal', title: 'sys.menu.terminal', icon: 'Monitor' },
    ],
  },
]

// ========== 路径 → 图标映射（标签页用） ==========
export const tabIconMap: Record<string, string> = Object.fromEntries(flatMenuItems.map((item) => [item.path, item.icon]))
