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
    task::JoinSet,
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
    let out_side_timer = Instant::now();
    let begin_to_fetch_year = year;
    dbg!(begin_to_fetch_year);

    let is_leap =
        |year: i32| -> usize { (year % 4 == 0 && (year % 400 == 0 || year % 100 != 0)) as usize };

    let (tx, mut rx) = mpsc::channel::<(i32, i32, String)>(CONCURRENT_DOWNLOAD_LIMIT * 2);

    // 请求tushare数据
    let client = Client::new();
    let semaphore = Arc::new(Semaphore::new(CONCURRENT_DOWNLOAD_LIMIT));
    let downloaded_file_counter = Arc::new(AtomicUsize::new(0));

    let mut tasks = Vec::new();

    tasks.extend(
        YEAR_DAYS[is_leap(year)]
            .iter()
            .enumerate()
            .flat_map(move |(m, &d)| {
                (1..=d).map(move |day| year * 10000 + (m as i32 + 1) * 100 + day)
            })
            .filter_map(|ymd| {
                let (y, m, d) = (ymd / 10000, (ymd / 100) % 100, ymd % 100);
                let y_adj = if m >= 3 { y } else { y - 1 };
                let ya = y_adj + y_adj / 4 - y_adj / 100 + y_adj / 400;
                let week_day_num = (ya + SAKAMOTO_WEEKDAY_ARRAY[(m - 1) as usize] + d) % 7;

                // 跳过周六日
                if week_day_num == 6 || week_day_num == 0 {
                    None
                } else {
                    Some((ymd, week_day_num))
                }
            })
            .map(|(ymd, week_day_num)| {
                let client = client.clone();
                let semaphore = Arc::clone(&semaphore);
                let tx = tx.clone();

                tokio::spawn(async move {
                    let _permit = semaphore.acquire().await.unwrap();

                    match client
                        .post(TUSHARE_URL)
                        .json(&json!({
                            "api_name": DAILY_API,
                            "token": TOKEN_RU,
                            "params": { "start_date": ymd, "end_date": ymd },
                            "fields": N_FIELDS
                        }))
                        .send()
                        .await
                    {
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
                })
            }),
    );

    // 多workers处理接收到的文件
    if !Path::new(folder_path).exists() {
        create_dir_all(folder_path)?;
    }

    let folder_path = folder_path.to_string();
    let counter = Arc::clone(&downloaded_file_counter);
    let sem = Arc::new(Semaphore::new(num_cpus::get()));
    let timer = Instant::now();

    let mut write_tasks = JoinSet::new();

    let dispatcher = tokio::spawn(async move {
        while let Some((ymd, week_day_num, text)) = rx.recv().await {
            let sem = Arc::clone(&sem);
            let counter = Arc::clone(&counter);
            let folder_path = folder_path.clone();

            write_tasks.spawn(async move {
                let _permit = sem.acquire_owned().await.unwrap();
                let file_path = format!("{}/rsps_{}_[{}]", folder_path, ymd, week_day_num);

                match File::create(&file_path).await {
                    Ok(mut file) => match file.write_all(text.as_bytes()).await {
                        Ok(_) => {
                            let count = counter.fetch_add(1, Ordering::SeqCst) + 1;
                            if count % 10 == 0 {
                                println!("fetched {} ---> {:?}", count, timer.elapsed());
                            }
                        }
                        Err(e) => {
                            eprintln!("写入失败 {}: {:?}", file_path, e);
                        }
                    },
                    Err(e) => {
                        eprintln!("创建文件失败 {}: {:?}", file_path, e);
                    }
                }
            });

            while let Some(res) = write_tasks.join_next().await {
                res.unwrap();
            }
        }
    });

    // 等待所有请求任务完成
    for task in tasks {
        let _ = task.await?; // 显式忽略返回值,避免编译器警告
    }

    drop(tx); // 关闭发送端(http请求)
    let _ = dispatcher.await;

    dbg!(downloaded_file_counter);
    dbg!(out_side_timer.elapsed());

    Ok(())
}
