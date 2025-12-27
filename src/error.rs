//! エラー型定義

use thiserror::Error;

/// PDF生成サービスのエラー型
#[derive(Error, Debug)]
pub enum PdfError {
    /// PDF生成エラー
    #[error("PDF生成エラー: {0}")]
    Generation(String),

    /// フォント読み込みエラー
    #[error("フォント読み込みエラー: {0}")]
    FontLoad(String),

    /// 印刷エラー
    #[error("印刷エラー: {0}")]
    Print(String),

    /// ファイルIOエラー
    #[error("ファイルIOエラー: {0}")]
    FileIO(#[from] std::io::Error),

    /// 設定エラー
    #[error("設定エラー: {0}")]
    Config(String),
}
