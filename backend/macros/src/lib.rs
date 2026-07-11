//! `ox-nginx-macros` —— OxNginx 项目的 proc-macro 集合。
//!
//! Rust 限制：标记了 `#[proc_macro_attribute]` 的函数**必须**位于 crate root，
//! 因此三个入口函数（`audit_log` / `check_permission` / `check_role`）本文件保留；
//! 实际的参数解析、宏体生成全部下沉到同名子模块：
//!
//! - [`audit_log`]         —— `#[audit_log(...)]` 的实现
//! - [`check_permission`]  —— `#[check_permission(...)]` 的实现
//! - [`check_role`]        —— `#[check_role(...)]` 的实现
//! - [`parser`]            —— 三个宏共享的参数解析（含 `value`/`mode`/`orRole`/`andRole`）

use proc_macro::TokenStream;

mod audit_log;
mod check_permission;
mod check_role;
mod parser;

/// 操作日志属性宏。详见 [`audit_log`]。
#[proc_macro_attribute]
pub fn audit_log(attr: TokenStream, item: TokenStream) -> TokenStream {
    audit_log::expand(attr, item)
}

/// 权限 + 角色双重校验。SaToken `@SaCheckPermission` 等价物。详见 [`check_permission`]。
#[proc_macro_attribute]
pub fn check_permission(attr: TokenStream, item: TokenStream) -> TokenStream {
    check_permission::expand(attr, item)
}

/// 角色校验。SaToken `@SaCheckRole` 等价物。详见 [`check_role`]。
#[proc_macro_attribute]
pub fn check_role(attr: TokenStream, item: TokenStream) -> TokenStream {
    check_role::expand(attr, item)
}
