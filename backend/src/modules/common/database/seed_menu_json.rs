//! 默认菜单种子 JSON loader
//! ponytail: 数据从 seed/menu.json 加载（编译期 include_str!）
//! 字段 p/n/t/i/u/y/pm/c 缩写减小文件体积——避免 JSON 字段名重复 96 次

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct MenuRow {
    /// 父菜单名（顶层为空串）
    #[serde(rename = "p")]
    pub p: String,
    /// 菜单名（数据库唯一）
    #[serde(rename = "n")]
    pub n: String,
    /// 标题 i18n key
    #[serde(rename = "t")]
    pub t: String,
    /// 图标
    #[serde(rename = "i")]
    pub i: String,
    /// 路由 path
    #[serde(rename = "u")]
    pub u: String,
    /// 类型 M=目录 / C=菜单 / F=按钮
    #[serde(rename = "y")]
    pub y: String,
    /// 权限标识
    #[serde(rename = "pm")]
    pub pm: String,
    /// 组件路径
    #[serde(rename = "c")]
    pub c: String,
}

#[derive(Debug, Deserialize)]
struct MenuFile {
    menus: Vec<MenuRow>,
}

/// ponytail: include_str! 编译期嵌入，运行时无文件 IO
pub fn load_default_menus() -> &'static [MenuRow] {
    let json = include_str!("seed/menu.json");
    let parsed: MenuFile = serde_json::from_str(json)
        .expect("seed/menu.json 格式错误，请检查");
    Box::leak(parsed.menus.into_boxed_slice())
}