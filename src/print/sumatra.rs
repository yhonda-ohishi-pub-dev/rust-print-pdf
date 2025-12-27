//! SumatraPDF連携モジュール
//!
//! SumatraPDFを使用してPDFを印刷

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::PdfError;

/// SumatraPDF プリンター
pub struct SumatraPrinter {
    /// SumatraPDFの実行ファイルパス
    sumatra_path: Option<PathBuf>,
}

impl SumatraPrinter {
    /// 新しいSumatraPrinterを作成
    pub fn new() -> Self {
        Self { sumatra_path: None }
    }

    /// SumatraPDFのパスを手動で設定
    pub fn with_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.sumatra_path = Some(path.into());
        self
    }

    /// SumatraPDFを検索
    pub fn find_sumatra(&mut self) -> Result<PathBuf, PdfError> {
        if let Some(ref path) = self.sumatra_path {
            if path.exists() {
                return Ok(path.clone());
            }
        }

        // 複数の場所でSumatraPDFを探す
        let search_paths = [".", "C:\\"];
        let candidates = [
            "SumatraPDF-3.5.2-64.exe",
            "SumatraPDF.exe",
        ];

        for search_path in &search_paths {
            for candidate in &candidates {
                let full_path = Path::new(search_path).join(candidate);
                if let Ok(abs_path) = std::fs::canonicalize(&full_path) {
                    if abs_path.exists() {
                        tracing::info!("SumatraPDF found: {:?}", abs_path);
                        self.sumatra_path = Some(abs_path.clone());
                        return Ok(abs_path);
                    }
                }
            }
        }

        // システムPATHでSumatraPDFを探す
        if let Ok(output) = Command::new("where").arg("SumatraPDF.exe").output() {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout);
                if let Some(first_line) = path_str.lines().next() {
                    let path = PathBuf::from(first_line.trim());
                    if path.exists() {
                        tracing::info!("SumatraPDF found in PATH: {:?}", path);
                        self.sumatra_path = Some(path.clone());
                        return Ok(path);
                    }
                }
            }
        }

        Err(PdfError::Print(
            "SumatraPDF実行ファイルが見つかりません".to_string(),
        ))
    }

    /// PDFを印刷
    ///
    /// # Arguments
    /// * `pdf_path` - 印刷するPDFファイルのパス
    /// * `printer_name` - プリンター名（None の場合はデフォルトプリンター）
    pub fn print(&self, pdf_path: &Path, printer_name: Option<&str>) -> Result<(), PdfError> {
        let sumatra_path = self.sumatra_path.as_ref().ok_or_else(|| {
            PdfError::Print("SumatraPDFのパスが設定されていません".to_string())
        })?;

        // PDFファイルの絶対パスを取得
        let abs_pdf_path = std::fs::canonicalize(pdf_path).map_err(|e| {
            PdfError::Print(format!("PDFファイルの絶対パス取得エラー: {}", e))
        })?;

        // SumatraPDFコマンドを構築
        let mut cmd = Command::new(sumatra_path);

        if let Some(printer) = printer_name {
            // 特定のプリンターに印刷
            cmd.arg("-print-to").arg(printer);
            tracing::info!("SumatraPDFで印刷中: {:?}, プリンター: {}", abs_pdf_path, printer);
        } else {
            // デフォルトプリンターに印刷
            cmd.arg("-print-to-default");
            tracing::info!("SumatraPDFで印刷中: {:?}, デフォルトプリンター", abs_pdf_path);
        }

        cmd.arg(&abs_pdf_path);

        // コマンド実行
        let output = cmd.output().map_err(|e| {
            PdfError::Print(format!("SumatraPDF実行エラー: {}", e))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(PdfError::Print(format!(
                "印刷エラー: ステータス={}, 出力={}",
                output.status, stderr
            )));
        }

        tracing::info!("印刷が正常に実行されました");
        Ok(())
    }

    /// 利用可能なプリンター一覧を取得
    pub fn list_printers() -> Result<Vec<String>, PdfError> {
        // PowerShellを使用してプリンター一覧を取得
        let output = Command::new("powershell")
            .args([
                "-Command",
                "Get-Printer | Select-Object -ExpandProperty Name",
            ])
            .output()
            .map_err(|e| PdfError::Print(format!("プリンター一覧取得エラー: {}", e)))?;

        if !output.status.success() {
            return Err(PdfError::Print("プリンター一覧の取得に失敗しました".to_string()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let printers: Vec<String> = stdout
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(printers)
    }

    /// デフォルトプリンターを取得
    pub fn get_default_printer() -> Result<Option<String>, PdfError> {
        let output = Command::new("powershell")
            .args([
                "-Command",
                "Get-CimInstance -ClassName Win32_Printer | Where-Object {$_.Default -eq $true} | Select-Object -ExpandProperty Name",
            ])
            .output()
            .map_err(|e| PdfError::Print(format!("デフォルトプリンター取得エラー: {}", e)))?;

        if !output.status.success() {
            return Ok(None);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let printer = stdout.trim().to_string();

        if printer.is_empty() {
            Ok(None)
        } else {
            Ok(Some(printer))
        }
    }
}

impl Default for SumatraPrinter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sumatra_printer_new() {
        let printer = SumatraPrinter::new();
        assert!(printer.sumatra_path.is_none());
    }

    #[test]
    fn test_sumatra_printer_with_path() {
        let printer = SumatraPrinter::new().with_path("C:\\SumatraPDF.exe");
        assert!(printer.sumatra_path.is_some());
    }

    #[test]
    #[ignore] // 実際のプリンターが必要
    fn test_list_printers() {
        let printers = SumatraPrinter::list_printers();
        println!("Printers: {:?}", printers);
    }
}
