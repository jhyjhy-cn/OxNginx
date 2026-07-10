use anyhow::Result;
use axum::http::header;
use axum::response::{IntoResponse, Response};
use axum::Json;
use rust_xlsxwriter::{Format, FormatAlign, Workbook};
use serde_json::json;

use crate::modules::common::dto::ApiResponse;

/// 单张 sheet 的内容：表头 + 行（每行已格式化为字符串）
pub struct Sheet {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

/// ponytail: 不泛型化——格式化（翻译、空值处理）每张表都不一样，调用方自己准备好
/// 列宽走默认（按表头长度估算），需要自定义就再加参数
pub fn build_xlsx(sheet_name: &str, sheet: &Sheet) -> Result<Vec<u8>> {
    let mut workbook = Workbook::new();
    let header_fmt = Format::new()
        .set_bold()
        .set_background_color("#4472C4")
        .set_font_color("#FFFFFF")
        .set_align(FormatAlign::Center);

    let ws = workbook.add_worksheet().set_name(sheet_name)?;

    for (col, h) in sheet.headers.iter().enumerate() {
        ws.write_string_with_format(0, col as u16, h, &header_fmt)?;
    }
    for (i, row) in sheet.rows.iter().enumerate() {
        let r = (i + 1) as u32;
        for (col, val) in row.iter().enumerate() {
            ws.write_string(r, col as u16, val)?;
        }
    }
    // 列宽按表头长度估算
    for (col, h) in sheet.headers.iter().enumerate() {
        let width = (h.chars().count() as f64 * 2.5).max(8.0).min(40.0);
        ws.set_column_width(col as u16, width)?;
    }

    let buf = workbook.save_to_buffer()?;
    Ok(buf)
}

/// 把 xlsx 字节包装成下载响应
pub fn xlsx_response(filename: String, buf: Vec<u8>) -> Response {
    (
        [
            (
                header::CONTENT_TYPE,
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string(),
            ),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{}\"", filename),
            ),
        ],
        buf,
    )
        .into_response()
}

/// 把导出过程中的任何错误转成标准 JSON 错误响应
pub fn export_error(e: anyhow::Error) -> Response {
    Json(json!(ApiResponse::<()>::error(e.to_string()))).into_response()
}
