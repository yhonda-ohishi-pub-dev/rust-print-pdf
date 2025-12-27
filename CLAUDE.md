# rust-print-pdf

出張旅費精算書PDF生成サービス (Go版 print_pdf の Rust移植)

## プロジェクト概要

- **目的**: Go版 `github.com/ohishi-yhonda-pub/print_pdf` をRustに完全移植
- **連携**: `rust-router/gateway` からライブラリとして参照される
- **参考**: `rust-scraper` と同様のパターン (tower::Service, Builder pattern)

## 依存関係

```toml
printpdf = "0.8"          # PDF生成 (Op-based API)
tokio = { version = "1", features = ["full"] }
tower = "0.4"             # Service trait
thiserror = "1"           # エラー型
tracing = "0.1"           # ログ
serde = { version = "1", features = ["derive"] }
```

## プロジェクト構成

```
src/
├── lib.rs              # モジュールエクスポート
├── models.rs           # Item, Ryohi, PrintRequest
├── config.rs           # PdfConfig (Builder pattern)
├── error.rs            # PdfError (thiserror)
├── traits.rs           # PdfGenerator trait
├── service.rs          # tower::Service実装
├── pdf/
│   ├── mod.rs
│   ├── generator.rs    # ReportLabStylePdfClient (PDF生成本体)
│   ├── fonts.rs        # Windowsフォント読み込み (yumin.ttf等)
│   ├── layout.rs       # A5レイアウト定数
│   └── text_utils.rs   # wrap_detail, wrap_kukan
└── print/
    ├── mod.rs
    └── sumatra.rs      # SumatraPDF連携
```

## 主要コンポーネント

### PdfService (tower::Service)

```rust
let mut service = PdfService::new();
let request = PdfRequest::new(items)
    .with_output_path("output.pdf")
    .with_print(true);
let result = service.call(request).await?;
```

### ReportLabStylePdfClient

- A5横 (210mm x 148mm) のPDF生成
- 日本語フォント対応 (yumin.ttf, yugothm.ttf, meiryo.ttc)
- printpdf 0.8 の Op-based API を使用

### SumatraPrinter

- SumatraPDF.exe を使用した印刷
- パス自動検索機能

## 重要な技術的決定

### printpdf 0.8 API

printpdf 0.8 は以前のバージョンと大きく異なる Op-based API を使用:

```rust
// フォント読み込み
let font = ParsedFont::from_bytes(&font_data, 0, &mut warnings)
    .ok_or_else(|| PdfError::FontLoad("エラー".to_string()))?;
let font_id = doc.add_font(&font);

// テキスト描画
ops.push(Op::SetFontSize { font: font_id.clone(), size: Pt(12.0) });
ops.push(Op::WriteText { items: vec![TextItem::Text(text)], font: font_id.clone() });

// 図形描画
ops.push(Op::DrawPolygon { polygon: Polygon { ... } });
ops.push(Op::DrawLine { line: Line { ... } });
```

## 引き継ぎ（2025-12-27）

### 完了した作業
- Phase 1-5: rust-print-pdf ライブラリ完全実装
- printpdf 0.8 API への対応 (Op-based API)
- 全テスト22件パス、警告なし
- examples/generate_test.rs でPDF生成確認 (76KB)
- Phase 6: rust-router/gateway への統合完了
  - Cargo.toml に `print-pdf-service = { path = "../../rust-print-pdf" }` 追加
  - proto/pdf.proto 追加 (PdfGenerator サービス定義)
  - grpc/pdf_service.rs 追加 (gRPC サービス実装)
  - main.rs に PdfGeneratorServer 追加
  - ビルド・テスト成功

### 未解決の問題
- なし

### gRPC エンドポイント

```protobuf
service PdfGenerator {
    rpc GeneratePdf(GeneratePdfRequest) returns (GeneratePdfResponse);
    rpc PrintPdf(PrintPdfRequest) returns (PrintPdfResponse);
    rpc Health(PdfHealthRequest) returns (PdfHealthResponse);
}
```

### 次のステップ
- プロジェクト完了（全フェーズ実装済み）
- 必要に応じて HTTP REST エンドポイントを追加可能
