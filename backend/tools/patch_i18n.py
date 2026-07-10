import json
base = r'D:\jhy\Work\jhyjhy.cn\OxNginx\backend\src\modules\common\database\language'

# 原前端 zh-CN.ts 里的 rbac/user 块（删除前）
zh_additions = {
    "sys.rbac.department": "部门",
    "sys.rbac.post": "岗位",
    "sys.rbac.posts": "岗位",
    "sys.rbac.roles": "角色",
    "sys.rbac.adminCannotReset": "不能重置超级管理员密码",
    "sys.rbac.adminCannotDisable": "不能禁用超级管理员",
    "sys.rbac.userCreateSuccess": "创建用户成功",
    "sys.rbac.userUpdateSuccess": "更新用户成功",
    "sys.rbac.confirmBatchResetPwd": "确认批量重置选中用户的密码为 123456？",
    "sys.rbac.confirmBatchDisable": "确认批量禁用选中用户？",
    "sys.rbac.confirmBatchEnable": "确认批量启用选中用户？",
    "sys.user.id": "编号",
    "sys.user.nickname": "昵称",
    "sys.user.phone": "手机号",
    "sys.user.email": "邮箱",
    "sys.user.gender": "性别",
    "sys.user.male": "男",
    "sys.user.female": "女",
    "sys.user.secret": "保密",
}

en_additions = {
    "sys.rbac.department": "Department",
    "sys.rbac.post": "Post",
    "sys.rbac.posts": "Post",
    "sys.rbac.roles": "Roles",
    "sys.rbac.adminCannotReset": "Cannot reset super admin password",
    "sys.rbac.adminCannotDisable": "Cannot disable super admin",
    "sys.rbac.userCreateSuccess": "User created",
    "sys.rbac.userUpdateSuccess": "User updated",
    "sys.rbac.confirmBatchResetPwd": "Reset selected users password to 123456?",
    "sys.rbac.confirmBatchDisable": "Disable selected users?",
    "sys.rbac.confirmBatchEnable": "Enable selected users?",
    "sys.user.id": "ID",
    "sys.user.nickname": "Nickname",
    "sys.user.phone": "Phone",
    "sys.user.email": "Email",
    "sys.user.gender": "Gender",
    "sys.user.male": "Male",
    "sys.user.female": "Female",
    "sys.user.secret": "Secret",
}

for name, add in (('zh-CN.json', zh_additions), ('en-US.json', en_additions)):
    p = f'{base}\\{name}'
    with open(p, 'r', encoding='utf-8') as f:
        data = json.load(f)
    miss = [k for k in add if k not in data]
    if miss:
        print(f'{name} 之前已存在: {set(add) - set(miss)}, 新增: {len(miss)}')
    data.update(add)
    with open(p, 'w', encoding='utf-8') as f:
        json.dump(data, f, ensure_ascii=False, indent=2, sort_keys=True)
        f.write('\n')
    print(f'{name}: 总 {len(data)} 条')
