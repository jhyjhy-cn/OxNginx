//! `#[audit_log(...)]` 的实现细节。
//!
//! `#[proc_macro_attribute]` 入口在 crate root 的 [`crate::audit_log`]，
//! 这里只放宏体展开函数。

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

use crate::parser::AuditArgs;

pub fn expand(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as AuditArgs);
    let func = parse_macro_input!(item as ItemFn);

    let vis = &func.vis;
    let sig = &func.sig;
    let body = &func.block;
    let attrs = &func.attrs;
    let module = &args.module;
    let action = &args.action;

    let capture_stmt = if let Some(cap) = &args.capture {
        quote! {
            if let Ok(__s) = serde_json::to_string(&(#cap)) {
                __ctx.params = Some(__s);
            }
        }
    } else {
        quote! {}
    };

    let expanded = quote! {
        #(#attrs)*
        #vis #sig {
            {
                let mut __ctx = ctx.0.lock();
                __ctx.module = Some(#module.to_string());
                __ctx.action = Some(#action.to_string());
                #capture_stmt
            }
            #body
        }
    };
    expanded.into()
}
