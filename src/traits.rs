//! PDF生成トレイト
//!
//! rust-scraperパターンを踏襲

use async_trait::async_trait;
use std::path::PathBuf;

use crate::error::PdfError;
use crate::models::Item;

/// PDF生成トレイト
#[async_trait]
pub trait PdfGenerator: Send + Sync {
    /// PDFを生成
    ///
    /// # Arguments
    /// * `items` - 精算書項目リスト
    ///
    /// # Returns
    /// 生成されたPDFファイルのパス
    async fn generate(&mut self, items: Vec<Item>) -> Result<PathBuf, PdfError>;

    /// PDFを生成して印刷
    ///
    /// # Arguments
    /// * `items` - 精算書項目リスト
    /// * `printer` - プリンター名（Noneの場合はデフォルトプリンター）
    ///
    /// # Returns
    /// 生成されたPDFファイルのパス
    async fn generate_and_print(
        &mut self,
        items: Vec<Item>,
        printer: Option<&str>,
    ) -> Result<PathBuf, PdfError>;
}
