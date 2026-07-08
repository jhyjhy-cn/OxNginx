import { JSEncrypt } from 'jsencrypt'
import api from '@/api'

let cachedPublicKey: string | null = null

async function getPublicKey(): Promise<string> {
  if (cachedPublicKey) return cachedPublicKey
  const { data } = await api.get('/api/auth/public-key')
  if (data.code !== 0) throw new Error('获取公钥失败')
  cachedPublicKey = data.data.public_key
  return cachedPublicKey!
}

export async function encryptPassword(password: string): Promise<string> {
  const pubKey = await getPublicKey()
  const encrypt = new JSEncrypt()
  encrypt.setPublicKey(`-----BEGIN PUBLIC KEY-----\n${pubKey}\n-----END PUBLIC KEY-----`)
  const encrypted = encrypt.encrypt(password)
  if (!encrypted) throw new Error('RSA 加密失败')
  return encrypted
}
