# rust-print-pdf

出張旅費精算書PDF生成サービス（Go版 `print_pdf` の Rust移植）

## 概要

このライブラリは、出張旅費精算書のPDFを生成し、オプションで印刷するサービスを提供します。
`rust-router/gateway` からライブラリとして参照され、gRPCエンドポイント経由で利用されます。

## 機能

- **PDF生成**: A5横サイズ（210mm x 148mm）の出張旅費精算書PDF
- **日本語フォント対応**: 游明朝、游ゴシック、メイリオ
- **印刷機能**: SumatraPDFを使用した自動印刷
- **非同期対応**: tower::Service traitによる非同期API

## インストール

```toml
[dependencies]
print-pdf-service = { path = "../rust-print-pdf" }
```

## 使用方法

### 基本的な使い方

```rust
use print_pdf_service::{PdfService, PdfRequest, Item, Ryohi};
use tower::ServiceExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // サービス作成
    let mut service = PdfService::new();

    // データ作成
    let item = Item {
        date: "12/25".to_string(),
        start_location: "東京駅".to_string(),
        end_location: "大阪駅".to_string(),
        detail: "新幹線のぞみ".to_string(),
        price: 14000,
        ryohi: Ryohi::default(),
    };

    // リクエスト作成
    let request = PdfRequest::new(vec![item])
        .with_output_path("output.pdf");

    // PDF生成
    let response = service.ready().await?.call(request).await?;
    println!("PDF generated: {:?}", response.output_path);

    Ok(())
}
```

### 印刷付きPDF生成

```rust
let request = PdfRequest::new(items)
    .with_output_path("output.pdf")
    .with_print(true);

let response = service.call(request).await?;
```

## データモデル

### Item（経費明細）

```rust
pub struct Item {
    pub date: String,           // 日付（例: "12/25"）
    pub start_location: String, // 出発地
    pub end_location: String,   // 到着地
    pub detail: String,         // 詳細（交通手段等）
    pub price: i32,             // 金額
    pub ryohi: Ryohi,           // 旅費情報
}
```

### Ryohi（旅費情報）

```rust
pub struct Ryohi {
    pub shucchou_date: String,  // 出張日
    pub kingaku: i32,           // 日当・宿泊費の合計金額
    pub bikou: String,          // 備考
}
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
│   ├── generator.rs    # ReportLabStylePdfClient
│   ├── fonts.rs        # Windowsフォント読み込み
│   ├── layout.rs       # A5レイアウト定数
│   └── text_utils.rs   # テキストユーティリティ
└── print/
    ├── mod.rs
    └── sumatra.rs      # SumatraPDF連携
```

## 依存関係

| クレート | バージョン | 用途 |
|---------|-----------|------|
| printpdf | 0.8 | PDF生成 |
| tokio | 1 | 非同期ランタイム |
| tower | 0.4 | Service trait |
| thiserror | 1 | エラー型定義 |
| serde | 1 | シリアライズ |

## 動作環境

- **OS**: Windows 10/11
- **フォント**: 游明朝（yumin.ttf）が必要
- **印刷**: SumatraPDF（同梱済み）

## 印刷機能

### SumatraPDF

プロジェクトには SumatraPDF 3.5.2 (64bit) が同梱されています。

```
bin/
└── SumatraPDF-3.5.2-64.exe
```

### 検索パス

SumatraPDF は以下の順序で自動検索されます:

1. `./bin/` ディレクトリ
2. `C:\Program Files\SumatraPDF`
3. `C:\Program Files (x86)\SumatraPDF`
4. 実行ファイルと同じディレクトリ
5. ユーザーの Downloads / Desktop
6. システム PATH

### 印刷付きPDF生成

```rust
use print_pdf_service::{PdfService, PdfRequest};
use tower::Service;

let mut service = PdfService::new();
let request = PdfRequest::new(items)
    .with_output_path("output.pdf")
    .with_print(true)                          // デフォルトプリンターに印刷
    .with_printer_name("Microsoft Print to PDF"); // プリンター指定（オプション）

let result = service.call(request).await?;
println!("印刷実行: {}", result.printed);
```

### プリンター操作

```rust
use print_pdf_service::print::SumatraPrinter;

// プリンター一覧取得
let printers = SumatraPrinter::list_printers()?;
for printer in &printers {
    println!("  - {}", printer);
}

// デフォルトプリンター取得
if let Some(default) = SumatraPrinter::get_default_printer()? {
    println!("デフォルト: {}", default);
}

// 手動で印刷
let mut printer = SumatraPrinter::new();
printer.find_sumatra()?;
printer.print(Path::new("output.pdf"), Some("Microsoft Print to PDF"))?;
```

## テスト

```bash
cargo test
```

## サンプル実行

```bash
# PDF生成のみ
cargo run --example generate_test

# 印刷テスト
cargo run --example print_test

# 実際に印刷（デフォルトプリンターに送信）
cargo run --example print_test -- --print

# プリンター一覧表示
cargo run --example print_test -- --list-printers
```

## gRPC連携

`rust-router/gateway` と連携してgRPCエンドポイントとして利用できます。

```protobuf
service PdfGenerator {
    rpc GeneratePdf(GeneratePdfRequest) returns (GeneratePdfResponse);
    rpc PrintPdf(PrintPdfRequest) returns (PrintPdfResponse);
    rpc Health(PdfHealthRequest) returns (PdfHealthResponse);
}
```

## ライセンス

MIT License
