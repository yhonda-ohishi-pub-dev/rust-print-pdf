//! PDF生成サービス
//!
//! tower::Serviceを実装したPDF生成サービス

use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::task::{Context, Poll};

use async_trait::async_trait;
use tower::Service;
use tracing::info;

use crate::config::PdfConfig;
use crate::error::PdfError;
use crate::models::Item;
use crate::pdf::generator::ReportLabStylePdfClient;
use crate::print::sumatra::SumatraPrinter;
use crate::traits::PdfGenerator;

/// PDF生成リクエスト
#[derive(Debug, Clone)]
pub struct PdfRequest {
    /// 精算書項目リスト
    pub items: Vec<Item>,
    /// 出力パス
    pub output_path: PathBuf,
    /// 印刷フラグ
    pub print: bool,
    /// プリンター名
    pub printer_name: Option<String>,
}

impl PdfRequest {
    /// 新しいPDF生成リクエストを作成
    pub fn new(items: Vec<Item>) -> Self {
        Self {
            items,
            output_path: PathBuf::from("travel_expense.pdf"),
            print: false,
            printer_name: None,
        }
    }

    /// 出力パスを設定
    pub fn with_output_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.output_path = path.into();
        self
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

/// PDF生成結果
#[derive(Debug, Clone)]
pub struct PdfResult {
    /// 生成されたPDFファイルのパス
    pub pdf_path: PathBuf,
    /// ファイルサイズ（バイト）
    pub file_size: u64,
    /// 印刷が実行されたか
    pub printed: bool,
}

impl PdfResult {
    /// 新しいPDF生成結果を作成
    pub fn new(pdf_path: PathBuf, printed: bool) -> std::io::Result<Self> {
        let metadata = std::fs::metadata(&pdf_path)?;
        Ok(Self {
            pdf_path,
            file_size: metadata.len(),
            printed,
        })
    }
}

/// tower::Serviceを実装したPDF生成サービス
#[derive(Debug, Clone, Default)]
pub struct PdfService {
    /// 設定
    config: PdfConfig,
}

impl PdfService {
    /// 新しいPDF生成サービスを作成
    pub fn new() -> Self {
        Self {
            config: PdfConfig::new(),
        }
    }

    /// 設定を指定してサービスを作成
    pub fn with_config(config: PdfConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl PdfGenerator for PdfService {
    async fn generate(&mut self, items: Vec<Item>) -> Result<PathBuf, PdfError> {
        let output_path = self.config.output_path.clone();

        // PDF生成は同期処理なのでtokio::task::spawn_blockingを使用
        let result = tokio::task::spawn_blocking(move || {
            let mut client = ReportLabStylePdfClient::new()
                .with_output_path(&output_path);
            client.generate(&items)
        })
        .await
        .map_err(|e| PdfError::Generation(format!("タスク実行エラー: {}", e)))??;

        Ok(result)
    }

    async fn generate_and_print(
        &mut self,
        items: Vec<Item>,
        printer: Option<&str>,
    ) -> Result<PathBuf, PdfError> {
        let pdf_path = self.generate(items).await?;

        let printer_name = printer.map(|s| s.to_string());
        let sumatra_path = self.config.sumatra_path.clone();
        let pdf_path_clone = pdf_path.clone();

        // 印刷も同期処理
        tokio::task::spawn_blocking(move || {
            let mut sumatra_printer = SumatraPrinter::new();
            if let Some(ref path) = sumatra_path {
                sumatra_printer = sumatra_printer.with_path(path);
            } else {
                sumatra_printer.find_sumatra()?;
            }
            sumatra_printer.print(&pdf_path_clone, printer_name.as_deref())
        })
        .await
        .map_err(|e| PdfError::Print(format!("タスク実行エラー: {}", e)))??;

        Ok(pdf_path)
    }
}

impl Service<PdfRequest> for PdfService {
    type Response = PdfResult;
    type Error = PdfError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: PdfRequest) -> Self::Future {
        info!("PDF生成リクエスト受信: items={}", req.items.len());

        let output_path = req.output_path.clone();
        let items = req.items.clone();
        let print = req.print;
        let printer_name = req.printer_name.clone();
        let sumatra_path = self.config.sumatra_path.clone();

        Box::pin(async move {
            // PDF生成
            let pdf_path = tokio::task::spawn_blocking(move || {
                let mut client = ReportLabStylePdfClient::new()
                    .with_output_path(&output_path);
                client.generate(&items)
            })
            .await
            .map_err(|e| PdfError::Generation(format!("タスク実行エラー: {}", e)))??;

            // 印刷が必要な場合
            let printed = if print {
                let pdf_path_clone = pdf_path.clone();
                let printer_name_clone = printer_name.clone();

                tokio::task::spawn_blocking(move || {
                    let mut printer = SumatraPrinter::new();
                    if let Some(ref path) = sumatra_path {
                        printer = printer.with_path(path);
                    } else {
                        printer.find_sumatra()?;
                    }
                    printer.print(&pdf_path_clone, printer_name_clone.as_deref())
                })
                .await
                .map_err(|e| PdfError::Print(format!("タスク実行エラー: {}", e)))??;

                true
            } else {
                false
            };

            let result = PdfResult::new(pdf_path, printed)?;

            info!(
                "PDF生成完了: path={:?}, size={}bytes, printed={}",
                result.pdf_path, result.file_size, result.printed
            );

            Ok(result)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdf_request_builder() {
        let items = vec![Item::default()];
        let req = PdfRequest::new(items)
            .with_output_path("/tmp/test.pdf")
            .with_print(true)
            .with_printer_name("MyPrinter");

        assert_eq!(req.output_path, PathBuf::from("/tmp/test.pdf"));
        assert!(req.print);
        assert_eq!(req.printer_name, Some("MyPrinter".to_string()));
    }

    #[test]
    fn test_pdf_service_new() {
        let service = PdfService::new();
        // デフォルトの出力パスは "./output"
        assert!(service.config.output_path.to_string_lossy().contains("output"));
    }
}
