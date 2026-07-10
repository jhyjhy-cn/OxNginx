import { getData, putData } from '@/api/http'
import type { Settings } from './type'

export const getSettings = () => getData<Settings>('/api/settings')

export const updateSettings = (payload: Settings) => putData('/api/settings', payload)