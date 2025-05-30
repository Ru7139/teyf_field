use std::{
    fs::{File, create_dir_all},
    io::Write,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Instant,
};

use serde_json::json;

const TUSHARE_URL: &str = "http://api.tushare.pro";
const DAILY_API: &str = "daily";
const TOKEN_RU: &str = "e1c23bbb77f2cc2ae0169d5f6da2b5b0df3b685763dad71085559c5a";
const TOKEN_FE: &str = "7ec7fdbb1c5d4c384becfdc5bcc0df6932503ea1a858dbf02196dabb";
const N_FIELDS: &str = "ts_code,trade_date,open,high,low,close,pre_close,change,pct_chg,vol,amount";

const YEAR_DAYS: [[i32; 12]; 2] = [
    [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31],
    [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31],
];

const SAKAMOTO_WEEKDAY_ARRAY: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];

#[rustfmt::skip]
pub async fn http_fetch_tushare_year_dayk_use_ru_token(
    year: i32,
    folder_path: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let done_fetch_timer = Instant::now();

    if !Path::new(folder_path).exists() { create_dir_all(folder_path).expect("Unable to create folder"); }

    let mut date_vec: Vec<i32> = Vec::with_capacity(366);

    date_vec.extend(
        YEAR_DAYS[(year % 4 == 0 && (year % 400 == 0 || year % 100 != 0)) as usize]
            .into_iter()
            .enumerate()
            .flat_map(|(m, d)| (1..=d).map(move |x: i32| year * 10000 + (m as i32 + 1) * 100 + x)),
    );

    let client = reqwest::Client::new();
    let downloaded_file_counter = Arc::new(AtomicUsize::new(0));

    type JHRst = tokio::task::JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>>;
    let mut tasks: Vec<JHRst> = Vec::with_capacity(366);
    let semaphore = Arc::new(tokio::sync::Semaphore::new(10usize));

    for ymd in date_vec.into_iter() {
        let dt = (ymd / 10000, (ymd / 100) % 100, ymd % 100); // date
        let y = if dt.1 >= 3 { dt.0 } else { dt.0 - 1 }; // year adjust
        let ya = y + y / 4 - y / 100 + y / 400;
        let week_day_num = (ya + SAKAMOTO_WEEKDAY_ARRAY[(dt.1 - 1) as usize] + dt.2) % 7;

        if week_day_num == 6 || week_day_num == 0 { continue; } // 如果为周六日则跳过

        let semaphore_clone = std::sync::Arc::clone(&semaphore); // 克隆所需的变量
        let client_clone = client.clone();
        let folder_path_clone = folder_path.to_string();
        let counter_clone = std::sync::Arc::clone(&downloaded_file_counter);

        let task = tokio::task::spawn(async move {
            let _permit = semaphore_clone.acquire().await.unwrap(); // 限制并发数量

            let response = client_clone
                .post(TUSHARE_URL)
                .json(&json!({
                    "api_name": DAILY_API,
                    "token": TOKEN_RU,
                    "params": json!({ "start_date": ymd, "end_date": ymd }),
                    "fields": N_FIELDS
                })).send().await.expect("Failed to send request");

            match response.status().is_success() {
                false => { return Err(format!("status code: {}", response.status()).into()); }
                true => {
                    let file_path = format!("{}/rsps_{}_[{}]", folder_path_clone, ymd, week_day_num);
                    let mut file = File::create(file_path).expect("Unable to create file");
                    file.write_all(response.text().await?.as_bytes()).expect("Unable to write");
                }
            }

            counter_clone.fetch_add(1, Ordering::SeqCst);

            Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
        });

        tasks.push(task);
    }

    for task in tasks { task.await??; }

    dbg!(downloaded_file_counter);
    dbg!(done_fetch_timer.elapsed());
    Ok(())
}
