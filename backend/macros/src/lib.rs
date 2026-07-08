use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

/// `#[operation_log("动作名")]` — 纯标记，不改代码，不注入任何东西
/// 操作日志由中间件自动记录（POST/PUT/DELETE）
#[proc_macro_attribute]
pub fn operation_log(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    quote! { #func }.into()
}
