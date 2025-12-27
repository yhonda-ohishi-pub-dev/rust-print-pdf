//! PDF生成モジュール
//!
//! - text_utils: テキスト折り返し・整形
//! - fonts: 日本語フォント読み込み
//! - layout: レイアウト定数
//! - generator: PDF生成ロジック

pub mod text_utils;
pub mod fonts;
pub mod layout;
pub mod generator;

pub use text_utils::{wrap_detail, wrap_kukan, align_rows, prepare_ryohi_for_print, RyohiPrintData, TextWrapResult};
pub use fonts::FontLoader;
pub use layout::*;
pub use generator::ReportLabStylePdfClient;
