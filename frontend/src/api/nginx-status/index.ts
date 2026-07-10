import { getData } from '@/api/http'
import type { NginxStatus } from './type'

export const getNginxStatus = () => getData<NginxStatus>('/api/nginx/status')