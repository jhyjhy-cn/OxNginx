use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, FnArg, ItemFn, LitStr, Pat, PatType};

/// `#[operation_log("动作名")]` — 包裹 handler，自动记录操作日志
#[proc_macro_attribute]
pub fn operation_log(attr: TokenStream, item: TokenStream) -> TokenStream {
    let action = parse_macro_input!(attr as LitStr);
    let func = parse_macro_input!(item as ItemFn);

    let vis = &func.vis;
    let sig = &func.sig;
    let attrs = &func.attrs;
    let block = &func.block;
    let fn_name = &sig.ident;

    let mut param_idents = Vec::new();
    let mut param_types = Vec::new();
    for arg in &sig.inputs {
        if let FnArg::Typed(PatType { pat, ty, .. }) = arg {
            if let Some(ident) = extract_ident(pat) {
                param_idents.push(ident);
                param_types.push(ty.clone());
            }
        }
    }

    // 闭包签名：原参数 + 额外的 req（让 Handler trait 类型匹配）
    let expanded = quote! {
        #(#attrs)*
        #vis async fn #fn_name(
            axum::extract::State(state): axum::extract::State<crate::AppState>,
            req: axum::extract::Request,
        ) -> axum::response::Response {
            use axum::response::IntoResponse;
            // 闭包：原参数 + req，body 用原函数体
            let handler = |#(#param_idents: #param_types,)* _req: axum::extract::Request| async move #block;

            let username = req
                .extensions()
                .get::<crate::middleware::TokenInfo>()
                .map(|t| t.username.clone());
            let method = req.method().clone();
            let uri = req.uri().to_string();

            let start = std::time::Instant::now();
            let resp = handler(axum::extract::State(state.clone()), req).await.into_response();
            let cost_ms = start.elapsed().as_millis() as i64;
            let ok = resp.status().is_success();

            if let Some(u) = username {
                let pool = state.db.pool().clone();
                let m = method.to_string();
                let s = if ok { "success" } else { "failed" }.to_string();
                let err = if !ok { Some(format!("HTTP {}", resp.status().as_u16())) } else { None };
                tokio::spawn(async move {
                    let _ = crate::service::log_service::log_operation(
                        &pool, &u, #action,
                        Some(&m), Some(&uri), None,
                        &s, Some(cost_ms), None, None, err.as_deref(),
                    ).await;
                });
            }

            resp
        }
    };

    TokenStream::from(expanded)
}

fn extract_ident(pat: &Pat) -> Option<syn::Ident> {
    match pat {
        Pat::Ident(pi) => Some(pi.ident.clone()),
        Pat::TupleStruct(ts) if ts.elems.len() == 1 => extract_ident(&ts.elems[0]),
        _ => None,
    }
}
