use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg, ItemFn, LitStr, Pat, PatType};

/// `#[operation_log("动作名")]` — 注入计时 + 操作日志到函数体
#[proc_macro_attribute]
pub fn operation_log(attr: TokenStream, item: TokenStream) -> TokenStream {
    let action = parse_macro_input!(attr as LitStr);
    let func = parse_macro_input!(item as ItemFn);

    let vis = &func.vis;
    let sig = &func.sig;
    let attrs = &func.attrs;
    let body = &func.block;

    // 找到 State<AppState> 参数的变量名
    let state_var = find_state_param(sig);

    let expanded = if let Some(sv) = state_var {
        quote! {
            #(#attrs)*
            #vis #sig {
                let #sv = #sv; // rebind to suppress unused warnings
                let __ol_pool = #sv.db.pool().clone();
                let __ol_start = std::time::Instant::now();
                let __ol_resp = (|| async move #body)().await;
                let __ol_cost = __ol_start.elapsed().as_millis() as i64;
                {
                    let a = #action.to_string();
                    tokio::spawn(async move {
                        let _ = crate::service::log_service::log_operation(
                            &__ol_pool, "system", &a, None, None, None,
                            "success", Some(__ol_cost), None, None, None,
                        ).await;
                    });
                }
                __ol_resp
            }
        }
    } else {
        // 没有 State<AppState> 参数，不注入日志
        quote! { #(#attrs)* #vis #sig #body }
    };

    TokenStream::from(expanded)
}

/// 从函数签名中找到 State<AppState> 参数的标识符
fn find_state_param(sig: &syn::Signature) -> Option<syn::Ident> {
    for arg in &sig.inputs {
        if let FnArg::Typed(PatType { pat, ty, .. }) = arg {
            // 检查类型是否是 State<AppState>
            if let syn::Type::Path(tp) = ty.as_ref() {
                if let Some(seg) = tp.path.segments.last() {
                    if seg.ident == "State" {
                        return extract_ident(pat);
                    }
                }
            }
        }
    }
    None
}

fn extract_ident(pat: &Pat) -> Option<syn::Ident> {
    match pat {
        Pat::Ident(pi) => Some(pi.ident.clone()),
        Pat::TupleStruct(ts) if ts.elems.len() == 1 => extract_ident(&ts.elems[0]),
        _ => None,
    }
}
