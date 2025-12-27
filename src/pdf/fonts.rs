//! 日本語フォント読み込み
//!
//! Windows環境の日本語フォントを読み込む

use std::path::PathBuf;
use crate::error::PdfError;

/// フォントローダー
pub struct FontLoader {
    /// フォントファイルパス
    font_path: Option<PathBuf>,
}

impl FontLoader {
    /// 新しいフォントローダーを作成
    pub fn new() -> Self {
        Self { font_path: None }
    }

    /// フォントを検索して読み込む
    ///
    /// 優先順位:
    /// 1. yumin.ttf (游明朝)
    /// 2. yugothm.ttf (游ゴシック)
    /// 3. meiryo.ttc (メイリオ)
    /// 4. msgothic.ttc (MSゴシック)
    pub fn find_font(&mut self) -> Result<PathBuf, PdfError> {
        let fonts_dir = get_windows_fonts_dir()?;

        // 優先順位順にフォントを検索
        let candidates = [
            "yumin.ttf",      // 游明朝
            "yugothm.ttf",    // 游ゴシック Medium
            "YuGothM.ttf",    // 游ゴシック Medium (大文字)
            "meiryo.ttc",     // メイリオ
            "msgothic.ttc",   // MSゴシック
            "msmincho.ttc",   // MS明朝
        ];

        for candidate in &candidates {
            let font_path = fonts_dir.join(candidate);
            if font_path.exists() {
                tracing::info!("フォント発見: {:?}", font_path);
                self.font_path = Some(font_path.clone());
                return Ok(font_path);
            }
        }

        Err(PdfError::FontLoad(
            "日本語フォントが見つかりません".to_string(),
        ))
    }

    /// フォントデータを読み込む
    pub fn load_font_data(&self) -> Result<Vec<u8>, PdfError> {
        let font_path = self.font_path.as_ref().ok_or_else(|| {
            PdfError::FontLoad("フォントが設定されていません".to_string())
        })?;

        std::fs::read(font_path).map_err(|e| {
            PdfError::FontLoad(format!("フォント読み込みエラー: {}", e))
        })
    }

    /// 現在のフォントパスを取得
    pub fn font_path(&self) -> Option<&PathBuf> {
        self.font_path.as_ref()
    }
}

impl Default for FontLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Windowsのフォントディレクトリを取得
fn get_windows_fonts_dir() -> Result<PathBuf, PdfError> {
    // WINDIR環境変数からフォントディレクトリを構築
    if let Ok(windir) = std::env::var("WINDIR") {
        let fonts_dir = PathBuf::from(windir).join("Fonts");
        if fonts_dir.exists() {
            return Ok(fonts_dir);
        }
    }

    // フォールバック: C:\Windows\Fonts
    let default_path = PathBuf::from("C:\\Windows\\Fonts");
    if default_path.exists() {
        return Ok(default_path);
    }

    Err(PdfError::FontLoad(
        "Windowsフォントディレクトリが見つかりません".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_windows_fonts_dir() {
        // Windows環境でのみテスト
        if cfg!(windows) {
            let result = get_windows_fonts_dir();
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_font_loader_find_font() {
        if cfg!(windows) {
            let mut loader = FontLoader::new();
            // フォントが見つかるかどうかは環境依存
            let _ = loader.find_font();
        }
    }
}
