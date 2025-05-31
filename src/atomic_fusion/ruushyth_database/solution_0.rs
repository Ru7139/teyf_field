use futures::{StreamExt, stream};
use num_cpus;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    fs::{create_dir_all, read_to_string},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Instant,
};
use tokio::{
    fs::File,
    io::AsyncWriteExt,
    sync::{Semaphore, mpsc},
    task::JoinSet,
};
use walkdir::WalkDir;

use surrealdb::{Surreal, engine::remote::ws::Ws, opt::auth::Root};

type SdbClient = surrealdb::engine::remote::ws::Client;

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

    let is_leap = |y: i32| -> usize { (y % 4 == 0 && (y % 400 == 0 || y % 100 != 0)) as usize };

    let (tx, mut rx) = mpsc::channel::<(i32, i32, String)>(CONCURRENT_DOWNLOAD_LIMIT * 2);

    // 请求tushare数据
    let client = reqwest::Client::new();
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

    // 若文件夹不存在，则创建
    if !Path::new(folder_path).exists() {
        create_dir_all(folder_path).expect("Unable to create parent folder or itself");
    }

    // 多workers处理接收到的文件
    let folder_path = folder_path.to_string();
    let counter = Arc::clone(&downloaded_file_counter);
    let sem = Arc::new(Semaphore::new(num_cpus::get()));

    let dispatcher = tokio::spawn(async move {
        let timer = Instant::now();
        let mut write_tasks = JoinSet::new();

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
                            if count % 20 == 0 {
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

// {
//     "request_id":"be79a9b4-f80f-4e76-8df4-c64d68d2bbf1",
//     "code":0,
//     "data":{
//         "fields":["ts_code","trade_date","open","high","low","close","pre_close","change","pct_chg","vol","amount"],
//         "items":[
//             ["600860.SH","20000104",6.68,6.9,6.6,6.79,6.57,0.22,3.35,5916.0,4003.081],
//             ["600701.SH","20000104",11.32,11.8,11.18,11.67,11.23,0.44,3.92,8644.0,10034.544],
//             ["000001.SZ","20000104",17.5,18.55,17.2,18.29,17.45,0.84,4.81,82161.0,147325.3568],
//             ...
//             ...
//         ]
//         "has_more":false,
//         "count":-1
//     },
//     "msg":""
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TushareData {
    request_id: String,
    code: i32,
    data: TushareInnerData,
    msg: String,
}

#[rustfmt::skip]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TushareInnerData {
    pub fields: Vec<String>,
    pub items: Vec<(String, String, f64, f64, f64, f64, Option<f64>, Option<f64>, Option<f64>, f64, Option<f64>)>,
    // "ts_code","trade_date","open","high","low","close","pre_close","change","pct_chg","vol","amount"
}

pub async fn deserialize_folder_tushare_file_to_vec(
    tushare_folder_path: &str,
) -> Result<Vec<TushareInnerData>, Box<dyn std::error::Error + Send + Sync>> {
    let ignored_files = [".DS_Store", "Thumbs.db"];

    let file_paths: Vec<PathBuf> = WalkDir::new(tushare_folder_path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry.file_type().is_file()
                && !ignored_files.contains(&entry.file_name().to_string_lossy().as_ref())
        })
        .map(|entry| entry.path().to_path_buf())
        .collect();

    let results: Vec<_> = file_paths
        .par_iter() // 使用 Rayon 多线程并行迭代
        .filter_map(|path| match read_to_string(path) {
            Ok(content) => match serde_json::from_str::<TushareData>(&content) {
                Ok(t_data) => Some(t_data.data),
                Err(e) => {
                    eprintln!("解析失败 {:?}: {}", path, e);
                    None
                }
            },
            Err(e) => {
                eprintln!("读取失败 {:?}: {}", path, e);
                None
            }
        })
        .collect();

    Ok(results)
}

#[rustfmt::skip]
pub async fn ws_root_signin_local_sdb(port: u16, user: &str,pass: &str)
-> Result<Surreal<SdbClient>, Box<dyn std::error::Error>> {
    let sdb: Surreal<SdbClient> = Surreal::new::<Ws>(format!("127.0.0.1:{}", port)).await?;
    sdb.signin(Root { username: user, password: pass }).await?;
    Ok(sdb)
}

#[rustfmt::skip]
pub async fn use_ns_db_record_tushareinner(
    sdb: &Surreal<SdbClient>, namespace: &str, database: &str, data: TushareInnerData, concurrent_limit: usize,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    sdb.use_ns(namespace).use_db(database).await?;

    futures::stream::iter(data.items)
        .map(|x| {
            let sdb = sdb.clone();
            let y = x.clone();
            async move { sdb.create((x.1, x.0)).content(SdbStockStruct::from(y)).await }
        })
        .buffer_unordered(concurrent_limit)
        .collect::<Vec<Result<Option<Record>, surrealdb::Error>>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;

    Ok(())
}

#[rustfmt::skip]
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SdbStockStruct { code: String, date: u32, open: f64, high: f64, low: f64, close: f64, pre_close: Option<f64>, change: Option<f64>, chg_percent: Option<f64>, vol: f64, amount: Option<f64> }

#[rustfmt::skip]
impl From<(String, String, f64, f64, f64, f64, Option<f64>, Option<f64>, Option<f64>, f64, Option<f64>)> for SdbStockStruct
{   // "ts_code","trade_date","open","high","low","close","pre_close","change","pct_chg","vol","amount"
    fn from(i: (String, String, f64, f64, f64, f64, Option<f64>, Option<f64>, Option<f64>, f64, Option<f64>))
    -> Self {
        let date_u32 = i.1.parse::<u32>().unwrap();
        SdbStockStruct {code: i.0, date: date_u32, open: i.2, high: i.3, low: i.4, close: i.5, pre_close: i.6, change: i.7, chg_percent: i.8, vol: i.9, amount: i.10}
    }
}

#[rustfmt::skip]
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Record { id: surrealdb::RecordId }
