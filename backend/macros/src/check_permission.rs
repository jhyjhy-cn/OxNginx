//! `#[check_permission(...)]` —— SaToken `@SaCheckPermission` 等价物。
//!
//! 形态：
//! - `#[check_permission("sys:user:add")]`
//! - `#[check_permission(value = ["sys:user:update","sys:user:delete"], mode = "OR")]`   // 默认 OR
//! - `#[check_permission(value = ["sys:user:view","sys:user:export"], mode = "AND")]`
//! - `#[check_permission(value = "user.add", orRole = "admin")]`            // 角色 OR
//! - `#[check_permission(value = "user.add", orRole = ["admin","staff"])]`
//! - `#[check_permission(value = "user.add", andRole = ["admin","manager"])]`
//!
//! 复合语义（与 SaToken 文档保持一致）：
//!   权限码 (OR/AND) 与 (orRole 任一命中) 与 (andRole 全部命中) 的"或"
//!   → 任一通道命中即放行。
//!
//! 拥有 `super_admin` 角色码的用户一律放行，不查权限表。
//! handler 必须显式声明 `token: axum::extract::Extension<crate::modules::common::middleware::TokenInfo>`，
//! 否则 rustc 报清晰错误。

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, spanned::Spanned, ItemFn};

use crate::parser::{CheckPermArgs, PermMode};

/// 校验：handler 是否有 `token: Extension<TokenInfo>` 参数。
/// 返回 `Err(spanned_error)` 时给出可定位错误（不走 `compile_error!`，避免 RA macro-error）。
pub(crate) fn require_token_arg(func: &ItemFn, macro_name: &str) -> Result<(), syn::Error> {
    let has_token = func.sig.inputs.iter().any(|arg| {
        if let syn::FnArg::Typed(pat_type) = arg {
            if let syn::Type::Path(tp) = &*pat_type.ty {
                if let Some(last) = tp.path.segments.last() {
                    if last.ident != "Extension" {
                        return false;
                    }
                    if let syn::PathArguments::AngleBracketed(ab) = &last.arguments {
                        for g in &ab.args {
                            if let syn::GenericArgument::Type(syn::Type::Path(itp)) = g {
                                if let Some(last2) = itp.path.segments.last() {
                                    if last2.ident == "TokenInfo" {
                                        return true;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        false
    });
    if !has_token {
        let msg = format!(
            "#[{macro_name}] requires the handler to declare `token: axum::extract::Extension<crate::modules::common::middleware::TokenInfo>`"
        );
        return Err(syn::Error::new(func.sig.span(), msg));
    }
    Ok(())
}

pub fn expand(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as CheckPermArgs);
    let func = parse_macro_input!(item as ItemFn);

    // 1) 编译期校验：handler 必须带 token 参数
    if let Err(e) = require_token_arg(&func, "check_permission") {
        return e.to_compile_error().into();
    }

    let vis = &func.vis;
    let sig = &func.sig;
    let body = &func.block;
    let attrs = &func.attrs;

    // 2) 序列化为 token 流所需字面量
    let perm_codes = args.perm_codes;
    let perm_mode_str: String = args.perm_mode.as_str().to_string();
    let role_or = args.role_or;
    let role_and = args.role_and;

    // 3) 生成检查逻辑
    // 3a) 权限码命中：perms 与 perm_codes 集合按 mode (OR/AND) 比较
    let perm_check = match args.perm_mode {
        PermMode::Or => quote! {
            __perm_codes.iter().any(|c| __perms.iter().any(|p| p == c))
        },
        PermMode::And => quote! {
            __perm_codes.iter().all(|c| __perms.iter().any(|p| p == c))
        },
    };

    // 3b) 角色 OR
    let role_or_check = if role_or.is_empty() {
        quote!(false)
    } else {
        quote! {
            __role_or.iter().any(|r| __roles.iter().any(|x| x == r))
        }
    };
    // 3c) 角色 AND
    let role_and_check = if role_and.is_empty() {
        quote!(false)
    } else {
        quote! {
            __role_and.iter().all(|r| __roles.iter().any(|x| x == r))
        }
    };

    let expanded = quote! {
        #(#attrs)*
        #vis #sig {
            // ponytail: 每个请求一次 DB 查询；大流量场景换 Arc<DashSet<String>> + token 失效回调，
            //          只改 user_dao::list_user_permissions 一处即可。
            {
                use axum::Json;
                use serde_json::json;
                use crate::modules::common::dto::ApiResponse;
                use crate::AppState;

                let __roles: Vec<String> = crate::modules::sys::dao::user_dao::list_enabled_role_codes_by_username(
                    state.db.pool(),
                    &token.username,
                )
                .await
                .unwrap_or_default();

                if !__roles.iter().any(|r| r == "super_admin") {
                    let __perms: Vec<String> = crate::modules::sys::dao::user_dao::list_user_permissions(
                        state.db.pool(),
                        &token.username,
                    )
                    .await
                    .unwrap_or_default();

                    let __perm_codes: &[&str] = &[#(#perm_codes),*];
                    let __perm_mode: &str = #perm_mode_str;
                    let __role_or: &[&str] = &[#(#role_or),*];
                    let __role_and: &[&str] = &[#(#role_and),*];

                    let __perm_hit: bool = #perm_check;
                    let __role_or_hit: bool = #role_or_check;
                    let __role_and_hit: bool = #role_and_check;

                    // 任一通道通过即放行（SaToken 语义）
                    if !(__perm_hit || __role_or_hit || __role_and_hit) {
                        return Json(json!(ApiResponse::<()>::error("无权限")));
                    }
                }
            }
            #body
        }
    };
    expanded.into()
}
