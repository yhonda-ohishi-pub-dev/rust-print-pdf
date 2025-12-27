//! データモデル定義
//!
//! Go版のmodels.goから移植

use serde::{Deserialize, Serialize};

/// 経費明細（旅費項目）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Ryohi {
    /// 日付 (YYYY-MM-DD形式)
    pub date: Option<String>,
    /// 日付配列（複数日の場合）
    #[serde(rename = "dateAr")]
    pub date_ar: Option<Vec<String>>,
    /// 行先
    pub dest: Option<String>,
    /// 行先配列
    #[serde(rename = "destAr")]
    pub dest_ar: Option<Vec<String>>,
    /// 摘要（詳細）
    #[serde(default)]
    pub detail: Vec<String>,
    /// 区間
    pub kukan: Option<String>,
    /// 区間分割
    #[serde(rename = "kukanSprit")]
    pub kukan_sprit: Option<Vec<String>>,
    /// 金額
    pub price: Option<i32>,
    /// 金額配列
    #[serde(rename = "priceAr")]
    pub price_ar: Option<Vec<i32>>,
    /// 数量
    pub vol: Option<f64>,
    /// 数量配列
    #[serde(rename = "volAr")]
    pub vol_ar: Option<Vec<f64>>,

    // 印刷用フィールド（PDF生成時に使用）
    /// 印刷用摘要
    #[serde(rename = "printDetail")]
    pub print_detail: Option<Vec<String>>,
    /// 印刷用摘要行数
    #[serde(rename = "printDetailRow")]
    pub print_detail_row: Option<i32>,
    /// 印刷用区間
    #[serde(rename = "printKukan")]
    pub print_kukan: Option<Vec<String>>,
    /// 印刷用区間行数
    #[serde(rename = "printKukanRow")]
    pub print_kukan_row: Option<i32>,
    /// 最大行数
    #[serde(rename = "maxRow")]
    pub max_row: Option<i32>,
    /// ページ数
    #[serde(rename = "pageCount")]
    pub page_count: Option<i32>,
}

/// 精算書項目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    /// 車両番号
    pub car: String,
    /// 氏名
    pub name: String,
    /// 目的
    pub purpose: Option<String>,
    /// 開始日 (YYYY-MM-DD形式)
    #[serde(rename = "startDate")]
    pub start_date: Option<String>,
    /// 終了日 (YYYY-MM-DD形式)
    #[serde(rename = "endDate")]
    pub end_date: Option<String>,
    /// 金額
    pub price: i32,
    /// 税額
    pub tax: Option<f64>,
    /// 説明
    pub description: Option<String>,
    /// 経費明細
    #[serde(default)]
    pub ryohi: Vec<Ryohi>,
    /// 所属
    pub office: Option<String>,
    /// 支払日 (YYYY/MM/DD形式)
    #[serde(rename = "payDay")]
    pub pay_day: Option<String>,
}

impl Default for Item {
    fn default() -> Self {
        Self {
            car: String::new(),
            name: String::new(),
            purpose: None,
            start_date: None,
            end_date: None,
            price: 0,
            tax: None,
            description: None,
            ryohi: Vec::new(),
            office: None,
            pay_day: None,
        }
    }
}

/// 印刷リクエスト
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintRequest {
    /// 精算書項目リスト
    pub items: Vec<Item>,
    /// 印刷フラグ
    #[serde(default)]
    pub print: bool,
    /// プリンター名
    #[serde(rename = "printerName")]
    pub printer_name: Option<String>,
}

impl PrintRequest {
    /// 新しい印刷リクエストを作成
    pub fn new(items: Vec<Item>) -> Self {
        Self {
            items,
            print: false,
            printer_name: None,
        }
    }

    /// 印刷フラグを設定
    pub fn with_print(mut self, print: bool) -> Self {
        self.print = print;
        self
    }

    /// プリンター名を設定
    pub fn with_printer_name(mut self, name: impl Into<String>) -> Self {
        self.printer_name = Some(name.into());
        self
    }
}

/// 金額をフォーマット（3桁区切り）
pub fn format_price(price: i32) -> String {
    let s = price.abs().to_string();
    let mut result = String::new();
    let chars: Vec<char> = s.chars().rev().collect();

    for (i, c) in chars.iter().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(*c);
    }

    if price < 0 {
        result.push('-');
    }

    result.chars().rev().collect()
}

/// 日付をパース (YYYY-MM-DD → YYYY年MM月DD日)
pub fn parse_date(date: &str) -> String {
    if date.is_empty() {
        return String::new();
    }

    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() == 3 {
        format!("{}年{}月{}日", parts[0], parts[1], parts[2])
    } else {
        date.to_string()
    }
}

/// 支払日をパース (YYYY/MM/DD → YYYY年MM月DD日)
pub fn parse_pay_day(date: &str) -> String {
    if date.is_empty() {
        return String::new();
    }

    let parts: Vec<&str> = date.split('/').collect();
    if parts.len() == 3 {
        format!("{}年{}月{}日", parts[0], parts[1], parts[2])
    } else {
        date.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_price() {
        assert_eq!(format_price(1000), "1,000");
        assert_eq!(format_price(12345), "12,345");
        assert_eq!(format_price(1234567), "1,234,567");
        assert_eq!(format_price(0), "0");
        assert_eq!(format_price(-1000), "-1,000");
    }

    #[test]
    fn test_parse_date() {
        assert_eq!(parse_date("2024-01-15"), "2024年01月15日");
        assert_eq!(parse_date(""), "");
    }

    #[test]
    fn test_parse_pay_day() {
        assert_eq!(parse_pay_day("2024/01/25"), "2024年01月25日");
        assert_eq!(parse_pay_day(""), "");
    }
}
