//! 設定管理

use std::path::PathBuf;

/// PDF生成サービスの設定
#[derive(Debug, Clone)]
pub struct PdfConfig {
    /// PDF出力ディレクトリ
    pub output_path: PathBuf,
    /// SumatraPDFの実行ファイルパス
    pub sumatra_path: Option<PathBuf>,
    /// ヘッドレスモード（印刷時にウィンドウを表示しない）
    pub headless: bool,
}

impl Default for PdfConfig {
    fn default() -> Self {
        Self {
            output_path: PathBuf::from("./output"),
            sumatra_path: None,
            headless: true,
        }
    }
}

impl PdfConfig {
    /// 新しい設定を作成
    pub fn new() -> Self {
        Self::default()
    }

    /// 出力パスを設定
    pub fn with_output_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.output_path = path.into();
        self
    }

    /// SumatraPDFのパスを設定
    pub fn with_sumatra_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.sumatra_path = Some(path.into());
        self
    }

    /// ヘッドレスモードを設定
    pub fn with_headless(mut self, headless: bool) -> Self {
        self.headless = headless;
        self
    }

    /// 環境変数から設定を読み込み
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(path) = std::env::var("PDF_OUTPUT_PATH") {
            config.output_path = PathBuf::from(path);
        }

        if let Ok(path) = std::env::var("SUMATRA_PDF_PATH") {
            config.sumatra_path = Some(PathBuf::from(path));
        }

        if let Ok(val) = std::env::var("PDF_HEADLESS") {
            config.headless = val.to_lowercase() != "false";
        }

        config
    }
}
