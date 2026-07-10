//! 通用枚举映射。数据库存 i32（0/1），业务代码用 enum 增强可读性。
//! 新增字段时把 0/1 换成有意义的 enum，不要在业务代码里裸用数字。

/// 操作日志结果 / 通用成败
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogStatus {
    Failed = 0,
    Success = 1,
}

impl LogStatus {
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

impl From<i32> for LogStatus {
    fn from(v: i32) -> Self {
        match v {
            1 => LogStatus::Success,
            _ => LogStatus::Failed,
        }
    }
}

/// 登录日志类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoginLogType {
    Logout = 0,
    Login = 1,
}

impl LoginLogType {
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

impl From<i32> for LoginLogType {
    fn from(v: i32) -> Self {
        match v {
            1 => LoginLogType::Login,
            _ => LoginLogType::Logout,
    }
}
}
