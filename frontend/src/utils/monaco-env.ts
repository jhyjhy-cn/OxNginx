// ponytail: monaco worker 注册。main.ts 顶层副作用导入一次，保证 ConfigEditor.vue 懒加载前环境就绪。
import * as monaco from 'monaco-editor'
import editorWorker from 'monaco-editor/esm/vs/editor/editor.worker?worker'

;(self as any).MonacoEnvironment = {
  getWorker(_workerId: string, label: string) {
    if (label === 'json') return new editorWorker()
    return new editorWorker()
  },
}

// nginx 语法高亮
monaco.languages.register({ id: 'nginx' })
monaco.languages.setMonarchTokensProvider('nginx', {
  keywords: [
    'server',
    'location',
    'upstream',
    'if',
    'else',
    'map',
    'geo',
    'http',
    'events',
    'stream',
    'mail',
    'types',
    'include',
    'worker_processes',
    'worker_connections',
    'keepalive_timeout',
    'proxy_pass',
    'proxy_set_header',
    'proxy_http_version',
    'proxy_cache',
    'proxy_cache_valid',
    'proxy_redirect',
    'return',
    'rewrite',
    'try_files',
    'root',
    'alias',
    'index',
    'listen',
    'server_name',
    'ssl_certificate',
    'ssl_certificate_key',
    'ssl_protocols',
    'ssl_ciphers',
    'ssl_prefer_server_ciphers',
    'access_log',
    'error_log',
    'log_format',
    'set',
    'break',
    'last',
    'permanent',
    'redirect',
    'expires',
    'add_header',
    'charset',
    'gzip',
    'gzip_types',
    'client_max_body_size',
    'sendfile',
    'tcp_nopush',
    'tcp_nodelay',
    'autoindex',
    'deny',
    'allow',
    'valid_referers',
    'ip_hash',
    'least_conn',
    'hash',
    'keepalive',
    'weight',
    'max_fails',
    'fail_timeout',
    'backup',
    'default_type',
    'send_timeout',
    'client_body_timeout',
    'fastcgi_pass',
    'fastcgi_param',
    'uwsgi_pass',
  ],
  tokenizer: {
    root: [
      [/#.*$/, 'comment'],
      [/\$[a-zA-Z_]\w*/, 'variable'],
      [/"([^"\\]|\\.)*"/, 'string'],
      [/'([^'\\]|\\.)*'/, 'string'],
      [/\b\d+[kmgs]?\b/, 'number'],
      [/[a-zA-Z_]\w*(?=\s*\{)/, 'type'],
      [
        /[a-zA-Z_]\w*/,
        {
          cases: { '@keywords': 'keyword', '@default': 'identifier' },
        },
      ],
      [/[{}()[\]]/, 'delimiter'],
      [/[;=]/, 'delimiter'],
    ],
  },
} as any)

export { monaco }
