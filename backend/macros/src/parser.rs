//! 各属性宏的参数解析，集中维护。
//!
//! 解析规则说明：
//! - 单字符串字面量作为"快捷式"（`#[check_permission("a")]`）
//! - `value = ["a","b"]` 是 SaToken 标准数组写法
//! - `orRole = "admin"` 或 `orRole = ["a","b"]` 二者皆可
//! - `andRole = ["a","b"]` 同上
//! - `mode = "OR" | "AND"` 仅对"同类型(权限码 / 角色码)命中"生效；orRole/andRole 与权限码之间始终是 OR

use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Expr, ExprArray, ExprLit, Lit, MetaNameValue, Token,
};

/// `#[audit_log(module = "...", action = "...", capture = expr)]`
pub struct AuditArgs {
    pub module: String,
    pub action: String,
    pub capture: Option<Expr>,
}

/// `#[check_permission(...)]`
///
/// 全部形态：
/// - `#[check_permission("sys:user:add")]`                                 // 单码
/// - `#[check_permission(value = "sys:user:add")]`                         // 与 SaToken 一致:value 单字符串也可
/// - `#[check_permission(value = ["a","b"], mode = "OR")]`                 // 默认 OR
/// - `#[check_permission(value = ["a","b"], mode = "AND")]`
/// - `#[check_permission(value = "user.add", orRole = "admin")]`
/// - `#[check_permission(value = "user.add", andRole = ["admin","manager"])]`
pub struct CheckPermArgs {
    /// `value` 解析后的权限码（单字符串/数组都归一化到 `perm_codes`）
    pub perm_codes: Vec<String>,
    pub perm_mode: PermMode,
    pub role_or: Vec<String>,
    pub role_and: Vec<String>,
}

/// `#[check_role(...)]`
///
/// 全部形态：
/// - `#[check_role("admin")]`
/// - `#[check_role(value = "admin")]`
/// - `#[check_role(value = ["a","b"], mode = "OR")]`              // 默认 OR
/// - `#[check_role(value = ["a","b"], mode = "AND")]`
pub struct CheckRoleArgs {
    pub role_codes: Vec<String>,
    pub mode: PermMode,
}

#[derive(Debug, Clone, Copy)]
pub enum PermMode {
    Or,
    And,
}

impl PermMode {
    pub fn parse(s: &str) -> syn::Result<Self> {
        match s {
            "OR" => Ok(PermMode::Or),
            "AND" => Ok(PermMode::And),
            _ => Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "mode must be \"OR\" or \"AND\"",
            )),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            PermMode::Or => "OR",
            PermMode::And => "AND",
        }
    }
}

// ============== 公共小工具 ==============

pub fn parse_str_lit(expr: &Expr) -> syn::Result<String> {
    if let Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) = expr {
        Ok(s.value())
    } else {
        Err(syn::Error::new_spanned(expr, "expected string literal"))
    }
}

/// 把 `Expr` 解释成"字符串数组"：
/// - 数组字面量 `["a","b"]`
/// - 单字符串 `"a"` -> `vec!["a".to_string()]`
pub fn parse_string_list(expr: &Expr, what: &str) -> syn::Result<Vec<String>> {
    if let Expr::Array(ExprArray { elems, .. }) = expr {
        let mut out = Vec::with_capacity(elems.len());
        for el in elems {
            if let Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) = el {
                out.push(s.value());
            } else {
                return Err(syn::Error::new_spanned(el, format!("{what} must be a list of string literals")));
            }
        }
        Ok(out)
    } else {
        // 退化成单字符串
        Ok(vec![parse_str_lit(expr)?])
    }
}

fn ident_of(nv: &MetaNameValue) -> syn::Result<String> {
    nv.path
        .get_ident()
        .map(|i| i.to_string())
        .ok_or_else(|| syn::Error::new_spanned(&nv.path, "expected ident"))
}

// ============== Parse 实现 ==============

impl Parse for AuditArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut module = None;
        let mut action = None;
        let mut capture = None;
        for nv in Punctuated::<MetaNameValue, Token![,]>::parse_terminated(input)? {
            let key = ident_of(&nv)?;
            match key.as_str() {
                "module" => module = Some(parse_str_lit(&nv.value)?),
                "action" => action = Some(parse_str_lit(&nv.value)?),
                "capture" => {
                    if matches!(&nv.value, Expr::Lit(ExprLit { lit: Lit::Str(_), .. })) {
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

impl Parse for CheckPermArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // (1) 快捷：`check_permission("sys:user:add")`
        if input.lookahead1().peek(Lit) {
            let lit: Lit = input.parse()?;
            if let Lit::Str(s) = lit {
                return Ok(CheckPermArgs {
                    perm_codes: vec![s.value()],
                    perm_mode: PermMode::Or,
                    role_or: vec![],
                    role_and: vec![],
                });
            }
            return Err(syn::Error::new_spanned(lit, "expected string literal"));
        }

        // (2) 名值对
        let mut perm_codes: Option<Vec<String>> = None;
        let mut perm_mode = PermMode::Or;
        let mut role_or: Vec<String> = vec![];
        let mut role_and: Vec<String> = vec![];
        for nv in Punctuated::<MetaNameValue, Token![,]>::parse_terminated(input)? {
            let key = ident_of(&nv)?;
            match key.as_str() {
                "value" => perm_codes = Some(parse_string_list(&nv.value, "value")?),
                "mode" => perm_mode = PermMode::parse(&parse_str_lit(&nv.value)?)?,
                "orRole" => role_or = parse_string_list(&nv.value, "orRole")?,
                "andRole" => role_and = parse_string_list(&nv.value, "andRole")?,
                _ => {
                    return Err(syn::Error::new_spanned(
                        &nv.path,
                        "unknown attr, allowed: value, mode, orRole, andRole",
                    ));
                }
            }
        }
        let perm_codes = perm_codes.ok_or_else(|| {
            syn::Error::new(input.span(), "missing `value = \"...\"` or `value = [\"...\", ...]`")
        })?;
        if perm_codes.is_empty() {
            return Err(syn::Error::new(input.span(), "`value` cannot be empty"));
        }
        Ok(CheckPermArgs { perm_codes, perm_mode, role_or, role_and })
    }
}

impl Parse for CheckRoleArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // (1) 快捷：`check_role("admin")`
        if input.lookahead1().peek(Lit) {
            let lit: Lit = input.parse()?;
            if let Lit::Str(s) = lit {
                return Ok(CheckRoleArgs {
                    role_codes: vec![s.value()],
                    mode: PermMode::Or,
                });
            }
            return Err(syn::Error::new_spanned(lit, "expected string literal"));
        }
        // (2) 名值对
        let mut role_codes: Option<Vec<String>> = None;
        let mut mode = PermMode::Or;
        for nv in Punctuated::<MetaNameValue, Token![,]>::parse_terminated(input)? {
            let key = ident_of(&nv)?;
            match key.as_str() {
                "value" => role_codes = Some(parse_string_list(&nv.value, "value")?),
                "mode" => mode = PermMode::parse(&parse_str_lit(&nv.value)?)?,
                _ => {
                    return Err(syn::Error::new_spanned(
                        &nv.path,
                        "unknown attr, allowed: value, mode",
                    ));
                }
            }
        }
        let role_codes = role_codes.ok_or_else(|| {
            syn::Error::new(input.span(), "missing `value = \"...\"` or `value = [\"...\", ...]`")
        })?;
        if role_codes.is_empty() {
            return Err(syn::Error::new(input.span(), "`value` cannot be empty"));
        }
        Ok(CheckRoleArgs { role_codes, mode })
    }
}
