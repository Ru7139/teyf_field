use serde_json::json;
use std::io::Write;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

const TUSHARE_URL: &str = "http://api.tushare.pro";
const DAILY_API: &str = "daily";
const TOKEN_RU: &str = "e1c23bbb77f2cc2ae0169d5f6da2b5b0df3b685763dad71085559c5a";
const NORMAL_FIELDS: &str =
    "ts_code, trade_date ,open, high, low, ,close, pre_close ,change, pct_chg, vol, amount";

const NORMAL_YEAR_DAYS: [i32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
const LEAP_YEAR_DAYS: [i32; 12] = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
const YEAR_DAYS: [[i32; 12]; 2] = [NORMAL_YEAR_DAYS, LEAP_YEAR_DAYS]; // false, ture
const WEEK_DAY_SAKAMOTO_ARRAY: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];

#[rustfmt::skip]
pub async fn download_every_stock_year_dayk_data(
    year: u32,
    folder_path: &str,
    concurrency_limit: usize,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let timer = std::time::Instant::now();

    // 函数简介
    // 参数：年份，下载文件夹，并发最大数量
    // 1. 检测文件夹是否存在，不存在则创建
    // 2. 计算是否为闰年，获得包含每天的Vec
    // 3. 并发获取每天的文件

    let f_path = std::path::Path::new(folder_path);
    if !f_path.exists() {
        std::fs::create_dir_all(f_path).expect("Unable to create folder or parent folder")
    }

    let y = year as i32;
    let mut date_vec: Vec<i32> = Vec::with_capacity(366);

    date_vec.extend(
        YEAR_DAYS[(y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)) as usize]
            .into_iter().enumerate()
            .flat_map(|(m, d)| (1..=d)
                .map(move |day| y * 10000 + (m as i32 + 1) * 100 + day)),
    );

    let counter = Arc::new(AtomicUsize::new(0));

    let mut tasks: Vec<tokio::task::JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>>> = Vec::new();
    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(concurrency_limit));
    let client = reqwest::Client::new();

    for ymd in date_vec.into_iter() {
        let dt = (ymd / 10000, (ymd / 100) % 100, ymd % 100); // date
        let y = if dt.1 >= 3 { dt.0 } else { dt.0 - 1 }; // year adjust
        let ya = y + y / 4 - y / 100 + y / 400; // part formula
        let week_day_num = (ya + WEEK_DAY_SAKAMOTO_ARRAY[(dt.1 - 1) as usize] + dt.2) % 7;

        if week_day_num == 6 || week_day_num == 0 { continue; } // 如果为周六日则跳过


        let semaphore_clone = std::sync::Arc::clone(&semaphore); // 克隆所需的变量
        let client_clone = client.clone();
        let folder_path_clone = folder_path.to_string();
        let counter_clone = Arc::clone(&counter);

        let task = tokio::task::spawn(async move {
            let _permit = semaphore_clone.acquire().await.unwrap(); // 获取信号量，限制并发数量

            let response = client_clone
                .post(TUSHARE_URL)
                .json(&json!({
                    "api_name": DAILY_API,
                    "token": TOKEN_RU,
                    "params": json!({ "start_date": ymd, "end_date": ymd }),
                    "fields": NORMAL_FIELDS
                }))
                .send()
                .await
                .expect("Failed to send request");

            match response.status().is_success() {
                false => { return Err(format!("status code: {}", response.status()).into()); }
                true => {
                    let file_path = format!("{}/rsps_{}_[{}]", folder_path_clone, ymd, week_day_num);
                    let mut file = std::fs::File::create(file_path).expect("Unable to create file");
                    file.write_all(response.text().await?.as_bytes()).expect("Unable to write");
                }
            }

            let current = counter_clone.fetch_add(1, Ordering::SeqCst) + 1;
            println!("{:3} ---> {:?}", current, timer.elapsed());

            Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
        });

        tasks.push(task);
    }

    for task in tasks {task.await??;} // 等待所有任务完成

    dbg!(timer.elapsed());
    Ok(())
}
