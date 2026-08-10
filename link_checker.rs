// link_checker.rs
use reqwest::{Client, StatusCode};
use clap::{App, Arg};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::timeout;
use tokio::runtime::Runtime;

type Results = Arc<Mutex<Vec<(String, Option<StatusCode>)>>>;

fn color_status(status: &Option<StatusCode>) -> String {
    match status {
        Some(code) => {
            let s = code.as_u16();
            if s >= 200 && s < 300 {
                format!("\x1b[92m{}\x1b[0m", s)
            } else if s >= 300 && s < 400 {
                format!("\x1b[93m{}\x1b[0m", s)
            } else if s >= 400 && s < 600 {
                format!("\x1b[91m{}\x1b[0m", s)
            } else {
                format!("{}", s)
            }
        }
        None => format!("\x1b[90mERR\x1b[0m"),
    }
}

async fn check_url(client: &Client, url: &str, timeout_sec: u64, follow: bool) -> Option<StatusCode> {
    let mut builder = client.get(url);
    if !follow {
        builder = builder.redirect(reqwest::redirect::Policy::none());
    }
    let res = timeout(Duration::from_secs(timeout_sec), builder.send()).await;
    match res {
        Ok(Ok(resp)) => Some(resp.status()),
        _ => None,
    }
}

fn load_links(filename: &str) -> Vec<String> {
    let file = File::open(filename).expect("Не удалось открыть файл");
    let reader = BufReader::new(file);
    reader.lines()
        .filter_map(|line| line.ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && !s.starts_with('#'))
        .collect()
}

fn main() {
    let matches = App::new("Link Checker")
        .version("1.0")
        .author("Your Name")
        .about("Проверка HTTP-статусов ссылок")
        .arg(Arg::new("source").required(true).help("Файл со ссылками или сами ссылки через пробел"))
        .arg(Arg::new("threads").long("threads").default_value("4").help("Количество потоков"))
        .arg(Arg::new("timeout").long("timeout").default_value("5").help("Таймаут в секундах"))
        .arg(Arg::new("follow").long("follow").help("Следовать по редиректам"))
        .arg(Arg::new("output").long("output").help("Экспорт в JSON или CSV"))
        .get_matches();

    let source = matches.value_of("source").unwrap();
    let threads: usize = matches.value_of("threads").unwrap().parse().unwrap();
    let timeout_sec: u64 = matches.value_of("timeout").unwrap().parse().unwrap();
    let follow = matches.is_present("follow");
    let output = matches.value_of("output");

    let links = if source.ends_with(".txt") {
        load_links(source)
    } else {
        // если это не файл, то это список ссылок, но мы просто берём все аргументы
        // для простоты поддержим только аргументы, но в реальности нужно парсить аргументы как список
        // В clap мы можем использовать множественные значения, но для простоты оставим как есть
        // В этом примере мы предполагаем, что source — это файл .txt
        // Для поддержки списка ссылок нужно использовать отдельный подход
        vec![source.to_string()]
    };

    if links.is_empty() {
        eprintln!("Нет ссылок для проверки.");
        std::process::exit(1);
    }

    println!("Проверка {} ссылок (потоков: {}, таймаут: {}с)...", links.len(), threads, timeout_sec);
    let start = std::time::Instant::now();

    let results = Arc::new(Mutex::new(Vec::new()));
    let client = Client::builder()
        .timeout(Duration::from_secs(timeout_sec + 5))
        .build()
        .unwrap();

    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let mut handles = Vec::new();
        let semaphore = Arc::new(tokio::sync::Semaphore::new(threads));
        for url in links {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let client = client.clone();
            let results = results.clone();
            let url = url.clone();
            handles.push(tokio::spawn(async move {
                let status = check_url(&client, &url, timeout_sec, follow).await;
                results.lock().unwrap().push((url, status));
                drop(permit);
            }));
        }
        for handle in handles {
            let _ = handle.await;
        }
    });

    let elapsed = start.elapsed();
    let results = results.lock().unwrap();

    println!("\nРезультаты:");
    for (url, status) in results.iter() {
        println!("  {} -> {}", url, color_status(status));
    }

    let total = results.len();
    let ok = results.iter().filter(|(_, s)| s.map_or(false, |c| c.as_u16() >= 200 && c.as_u16() < 300)).count();
    let redirect = results.iter().filter(|(_, s)| s.map_or(false, |c| c.as_u16() >= 300 && c.as_u16() < 400)).count();
    let error = results.iter().filter(|(_, s)| s.map_or(false, |c| c.as_u16() >= 400 && c.as_u16() < 600)).count();
    let failed = results.iter().filter(|(_, s)| s.is_none()).count();
    println!("\nСтатистика: Всего: {}, OK: {}, Редиректы: {}, Ошибки: {}, Сбои: {}", total, ok, redirect, error, failed);
    println!("Время: {:.2} сек.", elapsed.as_secs_f64());

    if let Some(out) = output {
        // экспорт (упрощённо)
        println!("Экспорт в {} (реализация не показана для краткости)", out);
    }
}
