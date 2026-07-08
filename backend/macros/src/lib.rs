use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
    ItemFn, Lit, MetaNameValue, Token,
};

struct AuditArgs {
    module: String,
    action: String,
    capture: Option<syn::Expr>,
}

impl Parse for AuditArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut module = None;
        let mut action = None;
        let mut capture = None;
        for nv in Punctuated::<MetaNameValue, Token![,]>::parse_terminated(input)? {
            let key = nv
                .path
                .get_ident()
                .ok_or_else(|| syn::Error::new_spanned(&nv.path, "expected ident"))?
                .to_string();
            match key.as_str() {
                "module" => module = Some(parse_str_lit(&nv.value)?),
                "action" => action = Some(parse_str_lit(&nv.value)?),
                "capture" => {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: Lit::Str(_),
                        ..
                    }) = &nv.value
                    {
                        return Err(syn::Error::new_spanned(
                            &nv.value,
                            "capture must be an expression (e.g. capture = req), not a string",
                        ));
                    }
                    capture = Some(nv.value);
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        &nv.path,
                        "unknown attr, allowed: module, action, capture",
                    ));
                }
            }
        }
        Ok(AuditArgs {
            module: module.ok_or_else(|| syn::Error::new(input.span(), "missing `module`"))?,
            action: action.ok_or_else(|| syn::Error::new(input.span(), "missing `action`"))?,
            capture,
        })
    }
}

fn parse_str_lit(expr: &syn::Expr) -> syn::Result<String> {
    if let syn::Expr::Lit(syn::ExprLit { lit: Lit::Str(s), .. }) = expr {
        Ok(s.value())
    } else {
        Err(syn::Error::new_spanned(expr, "expected string literal"))
    }
}

/// `#[audit_log(module = "...", action = "...", capture = expr)]`
/// - module: 英文 key（如 "site"）
/// - action: 中文标签（如 "创建站点"）
/// - capture: 可选，任意表达式（通常是 `Json<T>` 解出的 ident），会被 `serde_json::to_string` 序列化后存入 `params`
///
/// 宏**不改函数签名**。handler 必须显式声明 `ctx: axum::Extension<crate::audit::context::SharedAuditContext>` 参数。
#[proc_macro_attribute]
pub fn audit_log(attr: TokenStream, item: TokenStream) -> TokenStream {
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
                let mut __ctx = ctx.0.lock().unwrap();
                __ctx.module = Some(#module.to_string());
                __ctx.action = Some(#action.to_string());
                #capture_stmt
            }
            #body
        }
    };
    expanded.into()
}
