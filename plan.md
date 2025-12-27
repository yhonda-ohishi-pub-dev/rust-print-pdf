# print_pdf Go → Rust 移植計画

## 概要
GitHub: https://github.com/ohishi-yhonda-pub/print_pdf (Go言語)をRustに完全移植する。

**ターゲット**: `C:\rust\rust-print-pdf` (独立リポジトリ、rust-scraperと同様)
**参考プロジェクト**:
- `C:\rust\rust-router` (Windows Service, Auto-Update、このプロジェクトから参照される)
- `C:\rust\rust-scraper` (エラーハンドリング, Builder pattern、同様の構成)

## 連携方式

rust-scraperと同様に:
1. `C:\rust\rust-print-pdf` を独立したRustプロジェクトとして作成
2. `C:\rust\rust-router\gateway\Cargo.toml` から以下のように参照:
   ```toml
   # 開発時 (path依存)
   print-pdf-service = { path = "../rust-print-pdf" }

   # 本番時 (git依存)
   print-pdf-service = { git = "https://github.com/.../rust-print-pdf.git", branch = "main" }
   ```

---

## プロジェクト構成

```
C:\rust\rust-print-pdf\           # 独立リポジトリ (rust-scraperと同様)
├── Cargo.toml                    # ライブラリクレート
├── CLAUDE.md
├── README.md
├── src/
│   ├── lib.rs                    # ライブラリエクスポート
│   ├── models.rs                 # Item, Ryohi, PrintRequest
│   ├── config.rs                 # 設定 (from_env)
│   ├── error.rs                  # thiserror エラー型
│   ├── traits.rs                 # PdfService trait
│   ├── service.rs                # tower::Service実装
│   ├── pdf/
│   │   ├── mod.rs
│   │   ├── generator.rs          # ReportLabStylePdfClient
│   │   ├── fonts.rs              # 日本語フォント読み込み
│   │   ├── layout.rs             # テーブルレイアウト
│   │   └── text_utils.rs         # wrap_detail, wrap_kukan
│   └── print/
│       ├── mod.rs
│       └── sumatra.rs            # SumatraPDF連携
└── examples/
    └── generate_test.rs          # 単体テスト用サンプル
```

**HTTPサーバー・Windows Service・自動更新はrust-router/gateway側に実装**
- gatewayがprint-pdf-serviceをライブラリとして呼び出す
- HTTPエンドポイントはgatewayに追加

---

## 依存関係 (rust-print-pdf/Cargo.toml)

```toml
[package]
name = "print-pdf-service"
version = "0.1.0"
edition = "2021"
description = "出張旅費精算書PDF生成サービス"

[dependencies]
# PDF生成
printpdf = "0.8"

# 非同期
tokio = { version = "1", features = ["full"] }
async-trait = "0.1"

# tower Service
tower = "0.4"

# エラー
thiserror = "1"

# ログ
tracing = "0.1"

# シリアライズ
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[dev-dependencies]
tracing-subscriber = "0.3"
```

## gateway側の変更 (rust-router/gateway/Cargo.toml に追加)

```toml
# PDF生成サービス
print-pdf-service = { path = "../rust-print-pdf" }
```

---

## 実装フェーズ

### Phase 1: rust-print-pdf ライブラリ基盤

**src/models.rs** - データ構造
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ryohi {
    pub date: Option<String>,
    pub dest: Option<String>,
    pub detail: Vec<String>,
    pub kukan: Option<String>,
    pub price: Option<i32>,
    pub vol: Option<f64>,
    // ... 印刷用フィールド
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub car: String,
    pub name: String,
    pub purpose: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub price: i32,
    pub tax: Option<f64>,
    pub ryohi: Vec<Ryohi>,
    pub office: Option<String>,
    pub pay_day: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintRequest {
    pub items: Vec<Item>,
    #[serde(default)]
    pub print: bool,
    pub printer_name: Option<String>,
}
```

**src/error.rs** - エラー型 (rust-scraperパターン)
```rust
#[derive(Error, Debug)]
pub enum PdfError {
    #[error("PDF生成エラー: {0}")]
    Generation(String),
    #[error("フォント読み込みエラー: {0}")]
    FontLoad(String),
    #[error("印刷エラー: {0}")]
    Print(String),
    #[error("ファイルIOエラー: {0}")]
    FileIO(#[from] std::io::Error),
}
```

**src/config.rs** - 設定 (Builder pattern)
```rust
pub struct PdfConfig {
    pub output_path: PathBuf,
    pub sumatra_path: Option<PathBuf>,
}

impl PdfConfig {
    pub fn new() -> Self { ... }
    pub fn with_output_path(mut self, path: impl Into<PathBuf>) -> Self { ... }
    pub fn with_sumatra_path(mut self, path: impl Into<PathBuf>) -> Self { ... }
}
```

### Phase 2: テキストユーティリティ (src/pdf/text_utils.rs)

Go版の移植:
- `wrap_detail()` - 10文字制限
- `wrap_kukan()` - 22文字制限
- `align_rows()` - 複数行揃え
- `prepare_ryohi_for_print()` - 印刷用データ準備

### Phase 3: PDF生成 (src/pdf/)

1. **fonts.rs** - Windowsフォント読み込み
   - yumin.ttf, yugothm.ttf, meiryo.ttf
   - フォールバック機構

2. **layout.rs** - レイアウト定数
   - A5横: 210mm × 148mm
   - マージン: lX=10, tY=138, rX=200, bY=10
   - テーブル列幅: 30, 25, 28.75, 30, 30 mm

3. **generator.rs** - ReportLabStylePdfClient
   - `new(data: Vec<Item>)` - PDF生成
   - `draw_approval_table()` - 承認欄 (社長/会計/所属)
   - `draw_basic_info_table()` - 基本情報テーブル
   - `draw_main_data_table()` - 経費データテーブル (7行)
   - `draw_summary_table()` - 備考/計

### Phase 4: 印刷機能 (src/print/sumatra.rs)

- SumatraPDF.exe パス検索
- `std::process::Command` で印刷実行
- プリンター名指定対応

### Phase 5: トレイト・サービス (src/traits.rs, src/service.rs)

rust-scraperパターンを踏襲:
```rust
// src/traits.rs
#[async_trait]
pub trait PdfGenerator: Send + Sync {
    async fn generate(&self, items: Vec<Item>) -> Result<PathBuf, PdfError>;
    async fn generate_and_print(&self, items: Vec<Item>, printer: Option<&str>) -> Result<PathBuf, PdfError>;
}

// src/service.rs - tower::Service実装
impl Service<PdfRequest> for PdfService {
    type Response = PdfResult;
    type Error = PdfError;
    type Future = Pin<Box<dyn Future<Output = Result<...>> + Send>>;
}
```

### Phase 6: gateway側の変更 (rust-router/gateway)

1. **Cargo.toml追加**
   ```toml
   print-pdf-service = { path = "../rust-print-pdf" }
   ```

2. **HTTPエンドポイント追加** (既存のaxumルーターに統合)
   - POST /generate-pdf
   - POST /print-pdf
   - POST /print (multipart)

3. **gRPC サービス追加** (オプション)
   - PdfGeneratorService

---

## リスクと対策

| リスク | レベル | 対策 |
|--------|--------|------|
| 日本語フォント描画 | 高 | printpdf + TTF埋め込み、複数フォントフォールバック |
| レイアウト精度 | 中 | Go版の座標を正確に移植、目視比較テスト |
| Windows Service | 中 | rust-routerの実績あるパターンを流用 |
| SumatraPDF連携 | 低 | Command実行、パス検索実装 |

---

## 重要ファイル (参照用)

**元プロジェクト (Go)**:
- `reportlab_style_pdf.go` - PDF生成ロジック、座標定数
- `text_utils.go` - テキストラップ関数
- `models.go` - データ構造

**参考プロジェクト (Rust)**:
- `C:\rust\rust-router\gateway\src\main.rs` - Windows Service
- `C:\rust\rust-router\gateway\Cargo.toml` - 依存関係
- `C:\rust\rust-scraper\src\lib.rs` - ライブラリ構成
- `C:\rust\rust-scraper\src\error.rs` - thiserrorパターン
- `C:\rust\rust-scraper\src\traits.rs` - トレイト定義
- `C:\rust\rust-scraper\src\service.rs` - tower::Service実装

---

## 実装順序チェックリスト

### rust-print-pdf (独立ライブラリ)
- [x] Phase 1: Cargo.toml, src/lib.rs, src/models.rs, src/error.rs, src/config.rs
- [x] Phase 2: src/pdf/text_utils.rs
- [x] Phase 3: src/pdf/fonts.rs, src/pdf/layout.rs, src/pdf/generator.rs
- [x] Phase 4: src/print/sumatra.rs
- [x] Phase 5: src/traits.rs, src/service.rs
- [x] examples/generate_test.rs

### rust-router/gateway (既存プロジェクトへの追加)
- [x] Phase 6: Cargo.toml に print-pdf-service 依存追加
- [x] Phase 6: gRPC エンドポイント追加 (PdfGenerator サービス: GeneratePdf, PrintPdf, Health)
