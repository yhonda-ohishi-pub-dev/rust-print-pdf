//! 出張旅費精算書PDF生成サービス
//!
//! # 使用例
//!
//! ```rust,ignore
//! use print_pdf_service::{PdfService, PdfRequest, Item};
//! use tower::Service;
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut service = PdfService::new();
//!
//!     let request = PdfRequest::new(vec![item])
//!         .with_output_path("./output")
//!         .with_print(false);
//!
//!     let result = service.call(request).await.unwrap();
//!     println!("PDF generated: {:?}", result.pdf_path);
//! }
//! ```

pub mod config;
pub mod error;
pub mod models;
pub mod pdf;
pub mod print;
pub mod service;
pub mod traits;

// 主要な型をリエクスポート
pub use config::PdfConfig;
pub use error::PdfError;
pub use models::{Item, PrintRequest, Ryohi};
pub use print::SumatraPrinter;
pub use service::{PdfRequest, PdfResult, PdfService};
pub use traits::PdfGenerator;
