//! PDF レイアウト定数
//!
//! Go版のreportlab_style_pdf.goから移植した座標定数

/// A5横サイズ (mm)
pub const A5_WIDTH: f32 = 210.0;
pub const A5_HEIGHT: f32 = 148.0;

/// マージン (mm)
pub const MARGIN_LEFT: f32 = 10.0;
pub const MARGIN_TOP: f32 = 138.0;
pub const MARGIN_RIGHT: f32 = 200.0;
pub const MARGIN_BOTTOM: f32 = 10.0;

/// テーブル列幅 (mm)
pub const COL_WIDTH_DATE: f32 = 15.0;      // 月日
pub const COL_WIDTH_DEST: f32 = 14.0;      // 行先
pub const COL_WIDTH_DETAIL: f32 = 26.0;    // 摘要
pub const COL_WIDTH_KUKAN: f32 = 57.5;     // 区間
pub const COL_WIDTH_PRICE: f32 = 22.0;     // 金額
pub const COL_WIDTH_VOL: f32 = 13.0;       // 数量

/// テーブル列のX座標 (左端からの累積)
pub const COL_X_DATE: f32 = MARGIN_LEFT;
pub const COL_X_DEST: f32 = COL_X_DATE + COL_WIDTH_DATE;
pub const COL_X_DETAIL: f32 = COL_X_DEST + COL_WIDTH_DEST;
pub const COL_X_KUKAN: f32 = COL_X_DETAIL + COL_WIDTH_DETAIL;
pub const COL_X_PRICE: f32 = COL_X_KUKAN + COL_WIDTH_KUKAN;
pub const COL_X_VOL: f32 = COL_X_PRICE + COL_WIDTH_PRICE;

/// 行の高さ (mm)
pub const ROW_HEIGHT: f32 = 5.5;
pub const HEADER_ROW_HEIGHT: f32 = 6.0;

/// フォントサイズ (pt)
pub const FONT_SIZE_TITLE: f32 = 14.0;
pub const FONT_SIZE_HEADER: f32 = 10.0;
pub const FONT_SIZE_BODY: f32 = 8.0;
pub const FONT_SIZE_SMALL: f32 = 6.0;

/// 承認欄サイズ (mm)
pub const APPROVAL_WIDTH: f32 = 20.0;
pub const APPROVAL_HEIGHT: f32 = 15.0;

/// 承認欄のX座標 (右揃え)
pub const APPROVAL_X_PRESIDENT: f32 = MARGIN_RIGHT - APPROVAL_WIDTH;
pub const APPROVAL_X_ACCOUNTING: f32 = APPROVAL_X_PRESIDENT - APPROVAL_WIDTH;
pub const APPROVAL_X_DEPARTMENT: f32 = APPROVAL_X_ACCOUNTING - APPROVAL_WIDTH;

/// 基本情報テーブルのY座標
pub const INFO_TABLE_Y: f32 = 125.0;

/// データテーブルのY座標開始位置
pub const DATA_TABLE_Y_START: f32 = 105.0;

/// 1ページあたりの最大データ行数
pub const MAX_DATA_ROWS_PER_PAGE: usize = 7;

/// 摘要の最大文字数
pub const MAX_DETAIL_LENGTH: usize = 10;

/// 区間の最大文字数
pub const MAX_KUKAN_LENGTH: usize = 22;

/// ポイントをmmに変換
pub fn pt_to_mm(pt: f32) -> f32 {
    pt * 0.352778
}

/// mmをポイントに変換
pub fn mm_to_pt(mm: f32) -> f32 {
    mm / 0.352778
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pt_mm_conversion() {
        let mm = 10.0;
        let pt = mm_to_pt(mm);
        let back_to_mm = pt_to_mm(pt);
        assert!((mm - back_to_mm).abs() < 0.001);
    }

    #[test]
    fn test_column_positions() {
        // 列位置が正しく連続していることを確認
        assert!(COL_X_DEST > COL_X_DATE);
        assert!(COL_X_DETAIL > COL_X_DEST);
        assert!(COL_X_KUKAN > COL_X_DETAIL);
        assert!(COL_X_PRICE > COL_X_KUKAN);
        assert!(COL_X_VOL > COL_X_PRICE);
    }
}
