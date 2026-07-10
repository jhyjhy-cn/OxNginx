import { postData } from '@/api/http'

export const changePassword = (old_password: string, new_password: string) =>
  postData<string>('/api/change-password', { old_password, new_password })

export const changeUsername = (new_username: string, password: string) =>
  postData<string>('/api/change-username', { new_username, password })