export interface ChangePasswordResult {
  token?: string
  username?: string
  [key: string]: unknown
}

export interface ChangeUsernameResult {
  token: string
  username: string
}