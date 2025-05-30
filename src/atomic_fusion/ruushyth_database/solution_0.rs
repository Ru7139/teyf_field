use std::{
    fs::create_dir_all,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Instant,
};

use num_cpus;
use reqwest::Client;
use serde_json::json;
use tokio::{
    fs::File,
    io::AsyncWriteExt,
    sync::{Semaphore, mpsc},
};

const TUSHARE_URL: &str = "http://api.tushare.pro";
const DAILY_API: &str = "daily";
const TOKEN_RU: &str = "e1c23bbb77f2cc2ae0169d5f6da2b5b0df3b685763dad71085559c5a";
const N_FIELDS: &str = "ts_code,trade_date,open,high,low,close,pre_close,change,pct_chg,vol,amount";

const YEAR_DAYS: [[i32; 12]; 2] = [
    [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31],
    [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31],
];

const SAKAMOTO_WEEKDAY_ARRAY: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
const CONCURRENT_DOWNLOAD_LIMIT: usize = 10;

pub async fn http_fetch_tushare_year_dayk_use_ru_token(
    year: i32,
    folder_path: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let done_fetch_timer = Instant::now();
    dbg!("Begin to fetch");

    if !Path::new(folder_path).exists() {
        create_dir_all(folder_path)?;
    }

    let is_leap =
        |year: i32| -> usize { (year % 4 == 0 && (year % 400 == 0 || year % 100 != 0)) as usize };
    let mut date_vec: Vec<i32> = Vec::with_capacity(366);

    date_vec.extend(
        YEAR_DAYS[is_leap(year)]
            .into_iter()
            .enumerate()
            .flat_map(|(m, d)| (1..=d).map(move |x| year * 10000 + (m as i32 + 1) * 100 + x)),
    );

    let client = Client::new();
    let semaphore = Arc::new(Semaphore::new(CONCURRENT_DOWNLOAD_LIMIT));
    let (tx, mut rx) = mpsc::channel::<(i32, i32, String)>(CONCURRENT_DOWNLOAD_LIMIT * 2);
    let downloaded_file_counter = Arc::new(AtomicUsize::new(0));

    // 👷 启动单个调度器，它负责把任务发给多 worker
    let folder_path_clone = folder_path.to_string();
    let counter_clone = Arc::clone(&downloaded_file_counter);
    let num_workers = num_cpus::get();

    let dispatcher = tokio::spawn(async move {
        let sem = Arc::new(Semaphore::new(num_workers));
        let timer = done_fetch_timer.clone();
        while let Some((ymd, week_day_num, text)) = rx.recv().await {
            let sem_clone = sem.clone();
            let permit = sem_clone.acquire_owned().await.unwrap();
            let path = folder_path_clone.clone();
            let counter = Arc::clone(&counter_clone);

            tokio::spawn(async move {
                let file_path = format!("{}/rsps_{}_[{}]", path, ymd, week_day_num);
                match File::create(&file_path).await {
                    Ok(mut file) => {
                        if let Err(e) = file.write_all(text.as_bytes()).await {
                            eprintln!("写入失败 {}: {:?}", file_path, e);
                        } else {
                            let c_num = counter.fetch_add(1, Ordering::SeqCst);
                            if c_num % 10 == 0 {
                                println!("fetch {} ---> {:?}", c_num, timer.elapsed());
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("创建文件失败 {}: {:?}", file_path, e);
                    }
                }
                drop(permit); // 释放并发 worker 限制
            });
        }
    });

    // 📤 请求任务（受限于并发控制）
    let mut tasks = Vec::new();
    for ymd in date_vec {
        let dt = (ymd / 10000, (ymd / 100) % 100, ymd % 100);
        let y = if dt.1 >= 3 { dt.0 } else { dt.0 - 1 };
        let ya = y + y / 4 - y / 100 + y / 400;
        let week_day_num = (ya + SAKAMOTO_WEEKDAY_ARRAY[(dt.1 - 1) as usize] + dt.2) % 7;

        if week_day_num == 6 || week_day_num == 0 {
            continue;
        }

        let client = client.clone();
        let semaphore = Arc::clone(&semaphore);
        let tx = tx.clone();

        let task = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.unwrap();

            let response = client
                .post(TUSHARE_URL)
                .json(&json!({
                    "api_name": DAILY_API,
                    "token": TOKEN_RU,
                    "params": json!({ "start_date": ymd, "end_date": ymd }),
                    "fields": N_FIELDS
                }))
                .send()
                .await;

            match response {
                Ok(resp) if resp.status().is_success() => {
                    if let Ok(text) = resp.text().await {
                        let _ = tx.send((ymd, week_day_num, text)).await;
                    }
                }
                Ok(resp) => {
                    eprintln!("请求失败 {}: {:?}", ymd, resp.status());
                }
                Err(e) => {
                    eprintln!("网络错误 {}: {:?}", ymd, e);
                }
            }
        });

        tasks.push(task);
    }

    // 等待所有请求任务完成
    for task in tasks {
        let _ = task.await;
    }

    drop(tx); // 关闭发送端，让 dispatcher 退出
    let _ = dispatcher.await;

    dbg!(downloaded_file_counter);
    dbg!(done_fetch_timer.elapsed());

    Ok(())
}
