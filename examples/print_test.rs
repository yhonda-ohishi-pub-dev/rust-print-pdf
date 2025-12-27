//! 印刷機能テスト
//!
//! PDFを生成して印刷するサンプル
//!
//! 使用法:
//!   cargo run --example print_test
//!   cargo run --example print_test -- --print          # 実際に印刷
//!   cargo run --example print_test -- --list-printers  # プリンター一覧表示

use print_pdf_service::print::SumatraPrinter;
use print_pdf_service::{Item, PdfRequest, PdfService, Ryohi};
use tower::Service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ログ設定
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let args: Vec<String> = std::env::args().collect();

    // プリンター一覧表示モード
    if args.iter().any(|a| a == "--list-printers") {
        println!("=== プリンター一覧 ===");
        match SumatraPrinter::list_printers() {
            Ok(printers) => {
                for printer in &printers {
                    println!("  - {}", printer);
                }
                if printers.is_empty() {
                    println!("  (プリンターが見つかりません)");
                }
            }
            Err(e) => {
                eprintln!("エラー: {}", e);
            }
        }

        println!("\n=== デフォルトプリンター ===");
        match SumatraPrinter::get_default_printer() {
            Ok(Some(printer)) => println!("  {}", printer),
            Ok(None) => println!("  (デフォルトプリンターなし)"),
            Err(e) => eprintln!("エラー: {}", e),
        }

        return Ok(());
    }

    // SumatraPDF検索
    println!("=== SumatraPDF検索 ===");
    let mut printer = SumatraPrinter::new();
    match printer.find_sumatra() {
        Ok(path) => println!("SumatraPDF found: {:?}", path),
        Err(e) => {
            eprintln!("SumatraPDFが見つかりません: {}", e);
            eprintln!("bin/ ディレクトリにSumatraPDF-3.5.2-64.exeを配置してください");
        }
    }

    // サンプルデータ作成
    let items = vec![Item {
        car: "12-34".to_string(),
        name: "山田太郎".to_string(),
        purpose: Some("客先訪問".to_string()),
        start_date: Some("2024-12-25".to_string()),
        end_date: Some("2024-12-26".to_string()),
        price: 22510,
        tax: Some(2251.0),
        description: None,
        ryohi: vec![
            Ryohi {
                date: Some("2024-12-25".to_string()),
                dest: Some("大阪".to_string()),
                detail: vec!["新幹線のぞみ".to_string()],
                kukan: Some("東京駅　大阪駅".to_string()),
                price: Some(14000),
                vol: Some(1.0),
                ..Default::default()
            },
            Ryohi {
                date: Some("2024-12-25".to_string()),
                dest: Some("梅田".to_string()),
                detail: vec!["地下鉄".to_string()],
                kukan: Some("大阪駅　梅田".to_string()),
                price: Some(230),
                vol: Some(1.0),
                ..Default::default()
            },
            Ryohi {
                date: Some("2024-12-26".to_string()),
                dest: Some("なんば".to_string()),
                detail: vec!["地下鉄".to_string(), "日当・宿泊費".to_string()],
                kukan: Some("梅田　なんば".to_string()),
                price: Some(8280),
                vol: Some(1.0),
                ..Default::default()
            },
        ],
        office: Some("営業部".to_string()),
        pay_day: Some("2024/12/31".to_string()),
    }];

    // PDF生成サービス
    let mut service = PdfService::new();

    // 印刷フラグ確認
    let should_print = args.iter().any(|a| a == "--print");

    let output_path = "print_test_output.pdf";

    let request = PdfRequest::new(items)
        .with_output_path(output_path)
        .with_print(should_print);

    println!("\n=== PDF生成 ===");
    println!("出力パス: {}", output_path);
    println!(
        "印刷: {}",
        if should_print {
            "有効"
        } else {
            "無効 (--print で有効化)"
        }
    );

    match service.call(request).await {
        Ok(result) => {
            println!("\n=== 結果 ===");
            println!("生成ファイル: {:?}", result.pdf_path);
            println!("ファイルサイズ: {} bytes", result.file_size);
            println!("印刷実行: {}", result.printed);

            if result.printed {
                println!("\n印刷ジョブが送信されました。");
            }
        }
        Err(e) => {
            eprintln!("エラー: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}
