//! `#[check_role(...)]` —— SaToken `@SaCheckRole` 等价物。
//!
//! 形态：
//! - `#[check_role("admin")]`
//! - `#[check_role(value = ["admin","manager"], mode = "OR")]`   // 默认 OR
//! - `#[check_role(value = ["admin","manager","staff"], mode = "AND")]`
//!
//! 拥有 `super_admin` 角色码的用户一律放行。
//! handler 必须显式声明 `token: axum::extract::Extension<crate::modules::common::middleware::TokenInfo>`。

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

use crate::check_permission::require_token_arg;
use crate::parser::{CheckRoleArgs, PermMode};

pub fn expand(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as CheckRoleArgs);
    let func = parse_macro_input!(item as ItemFn);

    if let Err(e) = require_token_arg(&func, "check_role") {
        return e.to_compile_error().into();
    }

    let vis = &func.vis;
    let sig = &func.sig;
    let body = &func.block;
    let attrs = &func.attrs;

    let role_codes = args.role_codes;
    let role_check = match args.mode {
        PermMode::Or => quote! {
            __codes.iter().any(|c| __roles.iter().any(|r| r == c))
        },
        PermMode::And => quote! {
            __codes.iter().all(|c| __roles.iter().any(|r| r == c))
        },
    };

    let expanded = quote! {
        #(#attrs)*
        #vis #sig {
            {
                use axum::Json;
                use serde_json::json;
                use crate::modules::common::dto::ApiResponse;

                // ponytail: 每个请求一次 DB 查询；大流量场景换角色码缓存。
                let __roles: Vec<String> = crate::modules::sys::dao::user_dao::list_enabled_role_codes_by_username(
                    state.db.pool(),
                    &token.username,
                )
                .await
                .unwrap_or_default();

                if !__roles.iter().any(|r| r == "super_admin") {
                    let __codes: &[&str] = &[#(#role_codes),*];
                    if !#role_check {
                        return Json(json!(ApiResponse::<()>::error("无角色权限")));
                    }
                }
            }
            #body
        }
    };
    expanded.into()
}
