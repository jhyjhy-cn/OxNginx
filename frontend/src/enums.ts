/** 前端枚举常量（与后端 enum 对齐） */

/** 通用状态 / 菜单状态：1=启用 0=禁用 */
export const CommonStatus = {
  Disabled: 0,
  Enabled: 1,
} as const
export type CommonStatus = (typeof CommonStatus)[keyof typeof CommonStatus]

/** 操作日志结果：1=成功 0=失败 */
export const LogStatus = {
  Failed: 0,
  Success: 1,
} as const
export type LogStatus = (typeof LogStatus)[keyof typeof LogStatus]

/** 登录日志类型：1=登录 0=退出 */
export const LoginLogType = {
  Logout: 0,
  Login: 1,
} as const
export type LoginLogType = (typeof LoginLogType)[keyof typeof LoginLogType]

/** 菜单类型：1=目录 2=菜单 3=按钮 */
export const MenuType = {
  Directory: 1,
  Menu: 2,
  Button: 3,
} as const
export type MenuType = (typeof MenuType)[keyof typeof MenuType]

/** 用户禁用：0=启用 1=禁用（语义与 CommonStatus 相反） */
export const UserDisabled = {
  Enabled: 0,
  Disabled: 1,
} as const
export type UserDisabled = (typeof UserDisabled)[keyof typeof UserDisabled]
