"""
把前端 .vue/.ts 中所有 i18n key 字符串加上 sys. 前缀（业务 namespace）。

匹配场景：
  t('rbac.xxx') $t('rbac.xxx') $t("rbac.xxx")
  prop: 'rbac.xxx' / prop: "rbac.xxx"
  label: 'rbac.xxx'
  name: 'rbac.xxx'
  message: 'rbac.xxx'
  error('rbac.xxx') success('rbac.xxx') warning('rbac.xxx') info('rbac.xxx')
  confirm({ message: 'rbac.xxx' })
  ElMessageBox.confirm('rbac.xxx', ...) 之类的
  placeholder: 'rbac.xxx'

不动的 namespace：common / layout / login / forbidden
"""
import re, os

FRONTEND = {'common', 'layout', 'login', 'forbidden'}
ADD_SYS = {
    'menu', 'rbac', 'user', 'sites', 'dashboard', 'settings', 'upstreams',
    'logs', 'dict', 'log', 'access', 'templates', 'ssl', 'config',
    'files', 'terminal', 'theme', 'traffic', 'tabs', 'siteDetail',
    'dept', 'post', 'role',
}

# 匹配 'namespace.word' 或 "namespace.word"，namespace ∈ 业务集合
# namespace = 小写字母开头的小写字母数字串
NS = '|'.join(sorted(ADD_SYS, key=len, reverse=True))
pat = re.compile(r"(?<![A-Za-z0-9_.])('|\")(" + NS + r")(\.[A-Za-z0-9_]+)+('|\")")

# 也覆盖: function 形式  message: () => t('rbac.xxx')  已经通过 t() 匹配覆盖
# 还覆盖裸 i18n key：string literal 中独立出现 (前后不是 identifier char)

ROOTS = [r'D:\jhy\Work\jhyjhy.cn\OxNginx\frontend\src']
EXT = ('.vue', '.ts', '.tsx', '.js')

changed_files = 0
changed = 0
for root in ROOTS:
    for dp, _, fns in os.walk(root):
        for fn in fns:
            if not fn.endswith(EXT):
                continue
            p = os.path.join(dp, fn)
            with open(p, 'r', encoding='utf-8') as f:
                src = f.read()
            def sub(m):
                global changed
                q1, ns, rest, q2 = m.group(1), m.group(2), m.group(3), m.group(4)
                changed += 1
                return f'{q1}sys.{ns}{rest}{q2}'
            src2 = pat.sub(sub, src)
            if src2 != src:
                with open(p, 'w', encoding='utf-8') as f:
                    f.write(src2)
                changed_files += 1

print(f'改 {changed_files} 个文件, {changed} 处')
