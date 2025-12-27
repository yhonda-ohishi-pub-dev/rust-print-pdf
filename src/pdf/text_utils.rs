//! テキストユーティリティ
//!
//! Go版のtext_utils.goから移植
//! - wrap_detail: 摘要テキストの折り返し
//! - wrap_kukan: 区間テキストの折り返し
//! - align_rows: 行数の調整
//! - prepare_ryohi_for_print: 旅費データの印刷用準備

use regex::Regex;
use crate::models::{format_price, Ryohi};

/// テキスト折り返し結果
#[derive(Debug, Clone, Default)]
pub struct TextWrapResult {
    /// 折り返し後の行
    pub lines: Vec<String>,
    /// 行数
    pub row_count: usize,
}

impl TextWrapResult {
    /// 空の結果を作成
    pub fn empty() -> Self {
        Self {
            lines: Vec::new(),
            row_count: 0,
        }
    }

    /// 単一行の結果を作成
    pub fn single(line: String) -> Self {
        Self {
            lines: vec![line],
            row_count: 1,
        }
    }
}

/// 摘要テキストを指定文字数で折り返し
///
/// # Arguments
/// * `details` - 摘要文字列のスライス
/// * `max_len` - 1行あたりの最大文字数
///
/// # Returns
/// 折り返し結果
pub fn wrap_detail(details: &[String], max_len: usize) -> TextWrapResult {
    if details.is_empty() {
        return TextWrapResult::empty();
    }

    let mut result: Vec<String> = Vec::new();
    let mut current_line = String::new();

    for detail in details {
        // 区切り文字を考慮した新しい行の長さ
        let separator = if current_line.is_empty() { "" } else { "、" };
        let new_line_length = current_line.chars().count()
            + separator.chars().count()
            + detail.chars().count();

        if new_line_length <= max_len {
            // 全体が収まる場合
            current_line.push_str(separator);
            current_line.push_str(detail);
        } else {
            // 収まらない場合、現在の行が空でなければ確定して次の行に移る
            if !current_line.is_empty() {
                result.push(current_line);
            }

            // 新しい詳細項目を次の行に配置
            current_line = if detail.chars().count() > max_len {
                // 詳細項目自体が最大長を超える場合は切り詰め
                detail.chars().take(max_len).collect()
            } else {
                detail.clone()
            };
        }
    }

    // 最後の行を処理（空でない場合のみ）
    if !current_line.is_empty() {
        result.push(current_line);
    }

    // 空行を除去
    let filtered_result: Vec<String> = result
        .into_iter()
        .filter(|line| !line.trim().is_empty())
        .collect();

    let row_count = filtered_result.len();
    TextWrapResult {
        lines: filtered_result,
        row_count,
    }
}

/// 区間テキストを指定文字数で折り返し
///
/// # Arguments
/// * `kukan` - 区間文字列
/// * `max_len` - 1行あたりの最大文字数
///
/// # Returns
/// 折り返し結果
pub fn wrap_kukan(kukan: &str, max_len: usize) -> TextWrapResult {
    if kukan.is_empty() {
        return TextWrapResult::single(String::new());
    }

    // 特殊な文字列を置換
    let mut kukan = kukan.to_string();
    kukan = kukan.replace("_九州外空車適用", "　九州外空車適用");
    kukan = kukan.replace("適用*   追加", "適用*　追加");
    // 半角スペースを全角スペースに変換
    kukan = kukan.replace(' ', "　");

    // 区切り文字で分割 (全角スペース、｜、半角スペース+|、など)
    let re = Regex::new(r"[　｜]| \||\|").unwrap();
    let parts: Vec<&str> = re.split(&kukan).collect();

    let mut result: Vec<String> = Vec::new();
    let mut current_line = String::new();
    let mut current_count: usize = 0;

    for part in parts {
        let part_len = part.chars().count();

        if current_count != 0 && current_count + part_len == max_len {
            // ちょうど最大長になる場合
            result.push(format!("{}{}", current_line, part));
            current_line = String::new();
            current_count = 0;
        } else if part_len == max_len && current_line.is_empty() {
            // 単体で最大長の場合
            result.push(part.to_string());
            current_count = 0;
        } else if part_len > max_len {
            // 最大長を超える場合
            result.push("exceed*".to_string());
            current_count = 0;
        } else if current_count + part_len + 1 > max_len {
            // 現在行に追加すると最大長を超える場合
            if !current_line.is_empty() {
                result.push(current_line);
            }
            current_line = format!("{}　", part);
            current_count = part_len + 1;
        } else {
            // 現在行に追加できる場合
            current_count += part_len + 1;
            current_line.push_str(part);
            current_line.push('　');
        }
    }

    // 最後の行を処理
    if current_count != 0 {
        result.push(current_line);
    }

    // 前後の全角スペースを削除
    let result: Vec<String> = result
        .into_iter()
        .map(|line| {
            let line = line.replace(' ', "　");
            let line = line.trim_start_matches('　').to_string();
            line.trim_end_matches('　').to_string()
        })
        .collect();

    let row_count = result.len();
    TextWrapResult {
        lines: result,
        row_count,
    }
}

/// 他のデータ項目を最大行数に合わせて配列を調整
///
/// # Arguments
/// * `date` - 日付
/// * `dest` - 行先
/// * `price` - 金額
/// * `vol` - 数量
/// * `max_rows` - 最大行数
///
/// # Returns
/// (日付配列, 行先配列, 金額配列, 数量配列)
pub fn align_rows(
    date: Option<&str>,
    dest: Option<&str>,
    price: Option<i32>,
    vol: Option<f64>,
    max_rows: usize,
) -> (Vec<String>, Vec<String>, Vec<String>, Vec<String>) {
    let mut date_arr = vec![String::new(); max_rows];
    let mut dest_arr = vec![String::new(); max_rows];
    let mut price_arr = vec![String::new(); max_rows];
    let mut vol_arr = vec![String::new(); max_rows];

    // 最初の行に実際の値を設定
    if let Some(date_str) = date {
        // YYYY-MM-DD形式からMM/DD形式に変換
        if date_str.len() >= 10
            && date_str.chars().nth(4) == Some('-')
            && date_str.chars().nth(7) == Some('-')
        {
            let month = &date_str[5..7];
            let day = &date_str[8..10];
            date_arr[0] = format!("{}/{}", month, day);
        } else {
            date_arr[0] = date_str.to_string();
        }
    }

    if let Some(dest_str) = dest {
        dest_arr[0] = dest_str.to_string();
    }

    if let Some(price_val) = price {
        price_arr[0] = format_price(price_val);
    }

    if let Some(vol_val) = vol {
        vol_arr[0] = format!("{:.1}", vol_val);
    }

    (date_arr, dest_arr, price_arr, vol_arr)
}

/// 配列を最大行数まで拡張
fn extend_to_max_rows(lines: &[String], max_rows: usize) -> Vec<String> {
    // 空行を除去
    let filtered_lines: Vec<String> = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .cloned()
        .collect();

    // 必要に応じて最大行数まで拡張
    if filtered_lines.len() < max_rows {
        let mut result = vec![String::new(); max_rows];
        for (i, line) in filtered_lines.iter().enumerate() {
            result[i] = line.clone();
        }
        result
    } else {
        filtered_lines
    }
}

/// 旅費印刷用データ
#[derive(Debug, Clone, Default)]
pub struct RyohiPrintData {
    /// 日付行
    pub date_lines: Vec<String>,
    /// 行先行
    pub dest_lines: Vec<String>,
    /// 摘要行
    pub detail_lines: Vec<String>,
    /// 区間行
    pub kukan_lines: Vec<String>,
    /// 金額行
    pub price_lines: Vec<String>,
    /// 数量行
    pub vol_lines: Vec<String>,
    /// 最大行数
    pub max_rows: usize,
}

impl RyohiPrintData {
    /// 指定した行にコンテンツがあるかチェック
    pub fn has_content_in_row(&self, row: usize) -> bool {
        if row >= self.date_lines.len()
            && row >= self.dest_lines.len()
            && row >= self.detail_lines.len()
            && row >= self.kukan_lines.len()
            && row >= self.price_lines.len()
            && row >= self.vol_lines.len()
        {
            return false;
        }

        // いずれかの列にコンテンツがあればtrue
        if row < self.date_lines.len() && !self.date_lines[row].trim().is_empty() {
            return true;
        }
        if row < self.dest_lines.len() && !self.dest_lines[row].trim().is_empty() {
            return true;
        }
        if row < self.detail_lines.len() && !self.detail_lines[row].trim().is_empty() {
            return true;
        }
        if row < self.kukan_lines.len() && !self.kukan_lines[row].trim().is_empty() {
            return true;
        }
        if row < self.price_lines.len() && !self.price_lines[row].trim().is_empty() {
            return true;
        }
        if row < self.vol_lines.len() && !self.vol_lines[row].trim().is_empty() {
            return true;
        }

        false
    }

    /// 指定列のテキストを取得（範囲外の場合は空文字列）
    pub fn get_date(&self, row: usize) -> &str {
        self.date_lines.get(row).map(|s| s.as_str()).unwrap_or("")
    }

    pub fn get_dest(&self, row: usize) -> &str {
        self.dest_lines.get(row).map(|s| s.as_str()).unwrap_or("")
    }

    pub fn get_detail(&self, row: usize) -> &str {
        self.detail_lines.get(row).map(|s| s.as_str()).unwrap_or("")
    }

    pub fn get_kukan(&self, row: usize) -> &str {
        self.kukan_lines.get(row).map(|s| s.as_str()).unwrap_or("")
    }

    pub fn get_price(&self, row: usize) -> &str {
        self.price_lines.get(row).map(|s| s.as_str()).unwrap_or("")
    }

    pub fn get_vol(&self, row: usize) -> &str {
        self.vol_lines.get(row).map(|s| s.as_str()).unwrap_or("")
    }
}

/// 旅費データを印刷用に準備
///
/// # Arguments
/// * `ryohi` - 旅費データ
/// * `max_detail_len` - 摘要の最大文字数
/// * `max_kukan_len` - 区間の最大文字数
///
/// # Returns
/// 印刷用に整形されたデータ
pub fn prepare_ryohi_for_print(ryohi: &Ryohi, max_detail_len: usize, max_kukan_len: usize) -> RyohiPrintData {
    // 摘要を折り返し
    let detail_result = if !ryohi.detail.is_empty() {
        wrap_detail(&ryohi.detail, max_detail_len)
    } else {
        TextWrapResult::single(String::new())
    };

    // 区間を折り返し
    let kukan_result = if let Some(ref kukan) = ryohi.kukan {
        wrap_kukan(kukan, max_kukan_len)
    } else {
        TextWrapResult::single(String::new())
    };

    // 最大行数を決定
    let max_rows = detail_result.row_count.max(kukan_result.row_count).max(1);

    // 他のデータを最大行数に合わせる
    let (date_lines, dest_lines, price_lines, vol_lines) = align_rows(
        ryohi.date.as_deref(),
        ryohi.dest.as_deref(),
        ryohi.price,
        ryohi.vol,
        max_rows,
    );

    // すべての配列を最大行数に拡張
    let detail_lines = extend_to_max_rows(&detail_result.lines, max_rows);
    let kukan_lines = extend_to_max_rows(&kukan_result.lines, max_rows);

    RyohiPrintData {
        date_lines,
        dest_lines,
        detail_lines,
        kukan_lines,
        price_lines,
        vol_lines,
        max_rows,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_detail_empty() {
        let result = wrap_detail(&[], 10);
        assert_eq!(result.row_count, 0);
        assert!(result.lines.is_empty());
    }

    #[test]
    fn test_wrap_detail_single() {
        let details = vec!["テスト".to_string()];
        let result = wrap_detail(&details, 10);
        assert_eq!(result.row_count, 1);
        assert_eq!(result.lines[0], "テスト");
    }

    #[test]
    fn test_wrap_detail_multiple_fit_in_one_line() {
        let details = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        let result = wrap_detail(&details, 10);
        assert_eq!(result.row_count, 1);
        assert_eq!(result.lines[0], "A、B、C");
    }

    #[test]
    fn test_wrap_detail_multiple_lines() {
        let details = vec![
            "あいうえお".to_string(),
            "かきくけこ".to_string(),
            "さしすせそ".to_string(),
        ];
        let result = wrap_detail(&details, 10);
        // 「あいうえお、かきくけこ」は12文字なので収まらない
        assert!(result.row_count >= 2);
    }

    #[test]
    fn test_wrap_kukan_empty() {
        let result = wrap_kukan("", 22);
        assert_eq!(result.row_count, 1);
        assert_eq!(result.lines[0], "");
    }

    #[test]
    fn test_wrap_kukan_simple() {
        let result = wrap_kukan("東京　大阪", 22);
        assert!(result.row_count >= 1);
    }

    #[test]
    fn test_align_rows() {
        let (date, dest, price, vol) = align_rows(
            Some("2024-01-15"),
            Some("東京"),
            Some(1000),
            Some(1.5),
            3,
        );

        assert_eq!(date.len(), 3);
        assert_eq!(date[0], "01/15");
        assert_eq!(date[1], "");

        assert_eq!(dest[0], "東京");
        assert_eq!(price[0], "1,000");
        assert_eq!(vol[0], "1.5");
    }

    #[test]
    fn test_prepare_ryohi_for_print() {
        let ryohi = Ryohi {
            date: Some("2024-01-15".to_string()),
            dest: Some("東京".to_string()),
            detail: vec!["交通費".to_string(), "宿泊費".to_string()],
            kukan: Some("大阪　東京".to_string()),
            price: Some(10000),
            vol: Some(1.0),
            ..Default::default()
        };

        let print_data = prepare_ryohi_for_print(&ryohi, 10, 22);

        assert!(print_data.max_rows >= 1);
        assert_eq!(print_data.get_date(0), "01/15");
        assert_eq!(print_data.get_dest(0), "東京");
        assert!(!print_data.get_detail(0).is_empty() || !print_data.get_kukan(0).is_empty());
    }

    #[test]
    fn test_ryohi_print_data_has_content() {
        let data = RyohiPrintData {
            date_lines: vec!["01/15".to_string(), "".to_string()],
            dest_lines: vec!["東京".to_string(), "".to_string()],
            detail_lines: vec!["交通費".to_string(), "宿泊費".to_string()],
            kukan_lines: vec!["大阪　東京".to_string(), "".to_string()],
            price_lines: vec!["10,000".to_string(), "".to_string()],
            vol_lines: vec!["1.0".to_string(), "".to_string()],
            max_rows: 2,
        };

        assert!(data.has_content_in_row(0));
        assert!(data.has_content_in_row(1)); // detail_linesに「宿泊費」がある
        assert!(!data.has_content_in_row(10)); // 範囲外
    }
}
