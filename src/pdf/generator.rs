//! PDF生成ロジック
//!
//! Go版のreportlab_style_pdf.goから移植
//! printpdf 0.8クレートを使用してPDFを生成

use std::path::PathBuf;

use printpdf::*;

use crate::error::PdfError;
use crate::models::{format_price, Item};
use crate::pdf::fonts::FontLoader;
use crate::pdf::layout::*;
use crate::pdf::text_utils::prepare_ryohi_for_print;

/// ReportLabスタイルのPDF生成クライアント
pub struct ReportLabStylePdfClient {
    /// 出力パス
    output_path: PathBuf,
    /// フォントローダー
    font_loader: FontLoader,
}

impl ReportLabStylePdfClient {
    /// 新しいPDFクライアントを作成
    pub fn new() -> Self {
        Self {
            output_path: PathBuf::from("travel_expense_reportlab_style.pdf"),
            font_loader: FontLoader::new(),
        }
    }

    /// 出力パスを設定
    pub fn with_output_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.output_path = path.into();
        self
    }

    /// PDFを生成
    ///
    /// # Arguments
    /// * `items` - 精算書項目リスト
    ///
    /// # Returns
    /// 生成されたPDFファイルのパス
    pub fn generate(&mut self, items: &[Item]) -> Result<PathBuf, PdfError> {
        tracing::info!("Creating ReportLab Style PDF client...");

        // フォントを検索して読み込む
        self.font_loader.find_font()?;
        let font_data = self.font_loader.load_font_data()?;

        // ドキュメントを作成
        let mut doc = PdfDocument::new("出張旅費精算書");

        // フォントを追加
        let mut warnings = Vec::new();
        let font = ParsedFont::from_bytes(&font_data, 0, &mut warnings)
            .ok_or_else(|| PdfError::FontLoad("フォントパースエラー".to_string()))?;
        let font_id = doc.add_font(&font);

        // 各アイテムをページとして追加
        let mut pages = Vec::new();
        for (index, item) in items.iter().enumerate() {
            tracing::info!("Processing item {}/{}", index + 1, items.len());
            let ops = self.create_page_operations(&font_id, item);
            let page = PdfPage::new(Mm(A5_WIDTH), Mm(A5_HEIGHT), ops);
            pages.push(page);
        }

        // PDFを保存
        let bytes = doc
            .with_pages(pages)
            .save(&PdfSaveOptions::default(), &mut Vec::new());

        std::fs::write(&self.output_path, bytes)?;

        tracing::info!("ReportLab Style PDF saved successfully!");

        Ok(self.output_path.clone())
    }

    /// ページの操作を作成
    fn create_page_operations(&self, font_id: &FontId, item: &Item) -> Vec<Op> {
        let mut ops = Vec::new();

        // 外枠を描画
        self.add_outer_frame(&mut ops);

        // 承認テーブル（右上）
        self.add_approval_table(&mut ops, font_id);

        // 基本情報テーブル
        self.add_basic_info_table(&mut ops, font_id);

        // メインデータテーブル
        self.add_main_data_table(&mut ops, font_id);

        // 備考・計テーブル
        self.add_summary_table(&mut ops, font_id);

        // アイテム情報を印刷
        self.add_item_data(&mut ops, font_id, item);

        ops
    }

    /// 外枠を描画
    fn add_outer_frame(&self, ops: &mut Vec<Op>) {
        let start_x = 10.0;
        let start_y = 15.0;
        let end_x = A5_WIDTH - 10.0;
        let end_y = A5_HEIGHT - 10.0;

        ops.push(Op::SetOutlineThickness { pt: Pt(0.5) });
        ops.push(Op::SetOutlineColor {
            col: Color::Rgb(Rgb { r: 0.0, g: 0.0, b: 0.0, icc_profile: None }),
        });

        // 外枠を描画
        ops.push(Op::DrawPolygon {
            polygon: Polygon {
                rings: vec![PolygonRing {
                    points: vec![
                        LinePoint { p: Point::new(Mm(start_x), Mm(A5_HEIGHT - start_y)), bezier: false },
                        LinePoint { p: Point::new(Mm(end_x), Mm(A5_HEIGHT - start_y)), bezier: false },
                        LinePoint { p: Point::new(Mm(end_x), Mm(A5_HEIGHT - end_y)), bezier: false },
                        LinePoint { p: Point::new(Mm(start_x), Mm(A5_HEIGHT - end_y)), bezier: false },
                    ],
                }],
                mode: PaintMode::Stroke,
                winding_order: WindingOrder::NonZero,
            },
        });
    }

    /// 承認テーブルを描画
    fn add_approval_table(&self, ops: &mut Vec<Op>, font_id: &FontId) {
        let start_x = 155.0;
        let start_y = 25.0;
        let col_width = 15.0;
        let row_height1 = 5.0;
        let row_height2 = 15.0;

        ops.push(Op::SetOutlineThickness { pt: Pt(0.2) });

        // ヘッダー行
        let headers = ["社　長", "会　計", "所　属"];
        for (i, header) in headers.iter().enumerate() {
            let x = start_x + (i as f32) * col_width;

            // 矩形を描画
            self.add_rect(ops, x, start_y, col_width, row_height1);

            // テキストを描画
            self.add_text(ops, font_id, header, 9.0, x + 1.0, start_y + 4.0);
        }

        // データ行（空）
        for i in 0..3 {
            let x = start_x + (i as f32) * col_width;
            self.add_rect(ops, x, start_y + row_height1, col_width, row_height2);
        }
    }

    /// 基本情報テーブルを描画
    fn add_basic_info_table(&self, ops: &mut Vec<Op>, font_id: &FontId) {
        let start_x = 10.0;
        let start_y = 30.0;

        ops.push(Op::SetOutlineThickness { pt: Pt(0.2) });

        // 出発・帰着ラベル
        let row_height = 3.5;
        let diff_start_y = 3.0;

        self.add_text(ops, font_id, "出発", 9.0, start_x + 1.0, start_y + diff_start_y);
        self.add_text(ops, font_id, "　　月　　日", 9.0, start_x + 2.0, start_y + diff_start_y + row_height);
        self.add_text(ops, font_id, "帰着", 9.0, start_x + 1.0, start_y + diff_start_y + row_height * 2.0);
        self.add_text(ops, font_id, "　　月　　日", 9.0, start_x + 2.0, start_y + diff_start_y + row_height * 3.0);

        // テーブルヘッダー
        let headers = ["", "出張目的", "車両No.", "氏　名", "サイン"];
        let col_widths = [31.0, 25.0, 28.75, 30.0, 30.0];

        let mut current_x = start_x;
        for (i, header) in headers.iter().enumerate() {
            self.add_rect(ops, current_x, start_y, col_widths[i], 15.0);
            if !header.is_empty() {
                self.add_text(ops, font_id, header, 9.0, current_x + 1.0, start_y + 4.0);
            }
            current_x += col_widths[i];
        }
    }

    /// メインデータテーブルを描画
    fn add_main_data_table(&self, ops: &mut Vec<Op>, font_id: &FontId) {
        let start_x = 10.0;
        let start_y = 45.0;

        ops.push(Op::SetOutlineThickness { pt: Pt(0.2) });

        // 列幅
        let col_widths = [10.0, 17.0, 40.0, 30.0, 15.0, 15.0, 15.0, 25.0, 23.0];
        let row_height = 10.0;
        let header_height = 4.0;

        // ヘッダー
        let headers = ["日付", "行　先", "摘　　要", "区　　間", "交通機関", "運　賃", "特別料金", "旅費日当", "計"];

        let mut current_x = start_x;
        for (i, header) in headers.iter().enumerate() {
            self.add_rect(ops, current_x, start_y, col_widths[i], header_height);
            self.add_text(ops, font_id, header, 8.0, current_x + 1.0, start_y + 3.0);
            current_x += col_widths[i];
        }

        // データ行（7行）
        for row in 0..7 {
            current_x = start_x;
            let current_y = start_y + header_height + (row as f32) * row_height;

            for (col, &width) in col_widths.iter().enumerate() {
                if col == 2 {
                    // 摘要欄は左右の線のみ描画
                    self.add_vertical_line(ops, current_x, current_y, row_height);
                    self.add_vertical_line(ops, current_x + width, current_y, row_height);
                } else {
                    self.add_rect(ops, current_x, current_y, width, row_height);
                }
                current_x += width;
            }
        }
    }

    /// 備考・計テーブルを描画
    fn add_summary_table(&self, ops: &mut Vec<Op>, font_id: &FontId) {
        let start_x = 10.0;
        let start_y = 119.0;

        ops.push(Op::SetOutlineThickness { pt: Pt(0.2) });

        let col_widths = [145.0, 45.0];
        let row_height = 19.0;
        let headers = ["備考", "計"];

        let mut current_x = start_x;
        for (i, header) in headers.iter().enumerate() {
            self.add_rect(ops, current_x, start_y, col_widths[i], row_height);
            self.add_text(ops, font_id, header, 8.0, current_x + 2.0, start_y + 4.0);
            current_x += col_widths[i];
        }
    }

    /// アイテムデータを追加
    fn add_item_data(&self, ops: &mut Vec<Op>, font_id: &FontId, item: &Item) {
        self.add_base_data(ops, font_id, item);

        let start_x = 14.0;
        let start_y = 36.8;

        // 出発日
        if let Some(ref start_date) = item.start_date {
            if let Some(formatted) = format_date_mmdd(start_date) {
                self.add_text(ops, font_id, &formatted, 10.0, start_x, start_y);
            }
        }

        // 帰着日
        if let Some(ref end_date) = item.end_date {
            if let Some(formatted) = format_date_mmdd(end_date) {
                self.add_text(ops, font_id, &formatted, 10.0, start_x, start_y + 7.0);
            }
        }

        // 出張目的
        if let Some(ref purpose) = item.purpose {
            self.add_text(ops, font_id, purpose, 10.0, start_x + 32.0, start_y + 7.0);
        }

        // 車両
        if !item.car.is_empty() {
            self.add_text(ops, font_id, &item.car, 10.0, start_x + 52.0, start_y + 7.0);
        }

        // 氏名
        if !item.name.is_empty() {
            self.add_text(ops, font_id, &item.name, 10.0, start_x + 85.0, start_y + 7.0);
        }

        // 合計金額（上部の計欄）
        let price_str = format_price(item.price);
        self.add_text(ops, font_id, &price_str, 12.0, MARGIN_RIGHT - 30.0, MARGIN_TOP - 12.0);

        // 旅費データを処理
        self.add_ryohi_items(ops, font_id, &item.ryohi);
    }

    /// 基本データを描画
    fn add_base_data(&self, ops: &mut Vec<Op>, font_id: &FontId, item: &Item) {
        let start_x = 10.0;
        let start_y = 15.0;

        // タイトル
        let title = "出 張 旅 費 日 当 駐 車 料 込 精 算 書";
        self.add_text(ops, font_id, title, 14.0, start_x + 13.0, start_y + 5.0);

        // タイトル下線（2本）
        let title_width = 130.0;
        ops.push(Op::SetOutlineThickness { pt: Pt(0.3) });
        self.add_horizontal_line(ops, start_x + 13.0, start_y + 6.0, title_width);
        self.add_horizontal_line(ops, start_x + 13.0, start_y + 7.0, title_width);

        // 精算日
        if let Some(ref pay_day) = item.pay_day {
            if let Some(formatted) = format_pay_day_full(pay_day) {
                self.add_text(ops, font_id, &formatted, 9.0, start_x + 100.0, start_y + 5.0);
            }
        }

        // 所属（右上）
        if let Some(ref office) = item.office {
            self.add_text(ops, font_id, office, 10.0, start_x + 175.0, start_y + 5.0);
        }
    }

    /// 旅費データを印刷
    fn add_ryohi_items(
        &self,
        ops: &mut Vec<Op>,
        font_id: &FontId,
        ryohi_list: &[crate::models::Ryohi],
    ) {
        let start_x = 10.0;
        let start_y = 47.0;
        let col_widths = [10.0, 17.0, 40.0, 30.0, 15.0, 15.0, 15.0, 25.0, 23.0];
        let row_height = 10.0;

        let mut current_row: usize = 0;

        for (i, ryohi) in ryohi_list.iter().enumerate() {
            if current_row >= 14 {
                break;
            }

            // 旅費データを印刷用に準備
            let print_data = prepare_ryohi_for_print(ryohi, MAX_DETAIL_LENGTH, MAX_KUKAN_LENGTH);

            let remaining_rows = 14 - current_row;
            let actual_rows = print_data.max_rows.min(remaining_rows);

            let mut drawn_rows = 0;

            for row in 0..actual_rows {
                if !print_data.has_content_in_row(row) {
                    continue;
                }

                let logical_row = current_row + drawn_rows;
                let physical_row = logical_row / 2;
                let sub_row = logical_row % 2;
                let y_offset = (sub_row as f32) * 5.0;

                let current_y = start_y + (physical_row as f32) * row_height + y_offset;
                let mut current_x = start_x;

                // 日付
                let date = print_data.get_date(row);
                if !date.is_empty() {
                    self.add_text(ops, font_id, date, 10.0, current_x + 1.0, current_y + 6.0);
                }
                current_x += col_widths[0];

                // 行先
                let dest = print_data.get_dest(row);
                if !dest.is_empty() {
                    self.add_text(ops, font_id, dest, 10.0, current_x + 1.0, current_y + 6.0);
                }
                current_x += col_widths[1];

                // 摘要
                let detail = print_data.get_detail(row);
                if !detail.is_empty() {
                    self.add_text(ops, font_id, detail, 10.0, current_x + 1.0, current_y + 6.0);
                }
                current_x += col_widths[2];

                // 区間
                let kukan = print_data.get_kukan(row);
                if !kukan.is_empty() {
                    self.add_text(ops, font_id, kukan, 10.0, current_x + 1.0, current_y + 6.0);
                }
                current_x += col_widths[3];

                // 交通機関（空）
                current_x += col_widths[4];

                // 運賃（空）
                current_x += col_widths[5];

                // 特別料金（空）
                current_x += col_widths[6];

                // 旅費日当
                let price = print_data.get_price(row);
                if !price.is_empty() {
                    self.add_text(ops, font_id, price, 10.0, current_x + col_widths[7] - 15.0, current_y + 6.0);
                }
                current_x += col_widths[7];

                // 計
                let vol = print_data.get_vol(row);
                if !vol.is_empty() {
                    self.add_text(ops, font_id, vol, 10.0, current_x + col_widths[8] - 10.0, current_y + 6.0);
                }

                drawn_rows += 1;
            }

            current_row += drawn_rows;
            tracing::debug!(
                "旅費項目 {}: 最大行数={}, 実際印刷行数={}, 現在行={}",
                i + 1,
                print_data.max_rows,
                drawn_rows,
                current_row
            );
        }
    }

    /// テキストを追加
    fn add_text(&self, ops: &mut Vec<Op>, font_id: &FontId, text: &str, size: f32, x: f32, y: f32) {
        ops.push(Op::StartTextSection);
        ops.push(Op::SetTextCursor {
            pos: Point::new(Mm(x), Mm(A5_HEIGHT - y)),
        });
        ops.push(Op::SetFontSize {
            font: font_id.clone(),
            size: Pt(size),
        });
        ops.push(Op::SetLineHeight { lh: Pt(size) });
        ops.push(Op::SetFillColor {
            col: Color::Rgb(Rgb { r: 0.0, g: 0.0, b: 0.0, icc_profile: None }),
        });
        ops.push(Op::WriteText {
            items: vec![TextItem::Text(text.to_string())],
            font: font_id.clone(),
        });
        ops.push(Op::EndTextSection);
    }

    /// 矩形を描画
    fn add_rect(&self, ops: &mut Vec<Op>, x: f32, y: f32, width: f32, height: f32) {
        ops.push(Op::DrawPolygon {
            polygon: Polygon {
                rings: vec![PolygonRing {
                    points: vec![
                        LinePoint { p: Point::new(Mm(x), Mm(A5_HEIGHT - y)), bezier: false },
                        LinePoint { p: Point::new(Mm(x + width), Mm(A5_HEIGHT - y)), bezier: false },
                        LinePoint { p: Point::new(Mm(x + width), Mm(A5_HEIGHT - y - height)), bezier: false },
                        LinePoint { p: Point::new(Mm(x), Mm(A5_HEIGHT - y - height)), bezier: false },
                    ],
                }],
                mode: PaintMode::Stroke,
                winding_order: WindingOrder::NonZero,
            },
        });
    }

    /// 垂直線を描画
    fn add_vertical_line(&self, ops: &mut Vec<Op>, x: f32, y: f32, height: f32) {
        ops.push(Op::DrawLine {
            line: Line {
                points: vec![
                    LinePoint { p: Point::new(Mm(x), Mm(A5_HEIGHT - y)), bezier: false },
                    LinePoint { p: Point::new(Mm(x), Mm(A5_HEIGHT - y - height)), bezier: false },
                ],
                is_closed: false,
            },
        });
    }

    /// 水平線を描画
    fn add_horizontal_line(&self, ops: &mut Vec<Op>, x: f32, y: f32, width: f32) {
        ops.push(Op::DrawLine {
            line: Line {
                points: vec![
                    LinePoint { p: Point::new(Mm(x), Mm(A5_HEIGHT - y)), bezier: false },
                    LinePoint { p: Point::new(Mm(x + width), Mm(A5_HEIGHT - y)), bezier: false },
                ],
                is_closed: false,
            },
        });
    }
}

impl Default for ReportLabStylePdfClient {
    fn default() -> Self {
        Self::new()
    }
}

/// 日付をMM　DD形式にフォーマット
fn format_date_mmdd(date: &str) -> Option<String> {
    // YYYY-MM-DD形式を想定
    if date.len() >= 10 && date.chars().nth(4) == Some('-') && date.chars().nth(7) == Some('-') {
        let month = &date[5..7];
        let day = &date[8..10];
        Some(format!("{}　 {}", month, day))
    } else {
        Some(date.to_string())
    }
}

/// 支払日をフルフォーマット
fn format_pay_day_full(pay_day: &str) -> Option<String> {
    // YYYY/MM/DD or YYYY-MM-DD形式を想定
    let parts: Vec<&str> = if pay_day.contains('/') {
        pay_day.split('/').collect()
    } else {
        pay_day.split('-').collect()
    };

    if parts.len() == 3 {
        Some(format!("清算日　{}年 {}月 {}日", parts[0], parts[1], parts[2]))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_date_mmdd() {
        assert_eq!(format_date_mmdd("2024-01-15"), Some("01　 15".to_string()));
        assert_eq!(format_date_mmdd("invalid"), Some("invalid".to_string()));
    }

    #[test]
    fn test_format_pay_day_full() {
        assert_eq!(
            format_pay_day_full("2024/01/25"),
            Some("清算日　2024年 01月 25日".to_string())
        );
        assert_eq!(
            format_pay_day_full("2024-01-25"),
            Some("清算日　2024年 01月 25日".to_string())
        );
    }
}
