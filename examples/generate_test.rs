//! PDF生成テスト用サンプル
//!
//! 使用方法:
//! ```bash
//! cargo run --example generate_test
//! ```

use print_pdf_service::{Item, PdfRequest, PdfService, Ryohi};
use tower::Service;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ログ初期化
    tracing_subscriber::fmt::init();

    println!("=== PDF生成テスト ===");

    // テストデータ作成
    let items = create_test_items();

    println!("テストデータ: {} 件", items.len());

    // PDF生成サービス作成
    let mut service = PdfService::new();

    // PDF生成リクエスト
    let request = PdfRequest::new(items)
        .with_output_path("test_output.pdf")
        .with_print(false); // 印刷しない

    // PDF生成実行
    println!("PDF生成中...");
    let result = service.call(request).await?;

    println!("PDF生成完了!");
    println!("  ファイル: {:?}", result.pdf_path);
    println!("  サイズ: {} bytes", result.file_size);
    println!("  印刷: {}", result.printed);

    Ok(())
}

/// テストデータを作成
fn create_test_items() -> Vec<Item> {
    vec![
        Item {
            car: "12-34".to_string(),
            name: "山田太郎".to_string(),
            purpose: Some("客先訪問".to_string()),
            start_date: Some("2024-01-15".to_string()),
            end_date: Some("2024-01-16".to_string()),
            price: 25000,
            tax: Some(2500.0),
            description: None,
            ryohi: vec![
                Ryohi {
                    date: Some("2024-01-15".to_string()),
                    dest: Some("東京".to_string()),
                    detail: vec!["交通費".to_string(), "高速代".to_string()],
                    kukan: Some("福岡　東京".to_string()),
                    price: Some(15000),
                    vol: Some(1.0),
                    ..Default::default()
                },
                Ryohi {
                    date: Some("2024-01-16".to_string()),
                    dest: Some("福岡".to_string()),
                    detail: vec!["交通費".to_string()],
                    kukan: Some("東京　福岡".to_string()),
                    price: Some(10000),
                    vol: Some(1.0),
                    ..Default::default()
                },
            ],
            office: Some("営業部".to_string()),
            pay_day: Some("2024/01/25".to_string()),
        },
        Item {
            car: "56-78".to_string(),
            name: "鈴木花子".to_string(),
            purpose: Some("研修参加".to_string()),
            start_date: Some("2024-01-20".to_string()),
            end_date: Some("2024-01-20".to_string()),
            price: 8000,
            tax: Some(800.0),
            description: None,
            ryohi: vec![Ryohi {
                date: Some("2024-01-20".to_string()),
                dest: Some("大阪".to_string()),
                detail: vec!["交通費".to_string(), "宿泊費".to_string()],
                kukan: Some("福岡　大阪".to_string()),
                price: Some(8000),
                vol: Some(1.0),
                ..Default::default()
            }],
            office: Some("開発部".to_string()),
            pay_day: Some("2024/01/31".to_string()),
        },
    ]
}
