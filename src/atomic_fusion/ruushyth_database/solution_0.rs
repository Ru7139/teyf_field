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

use rust_decimal::{Decimal, prelude::FromPrimitive};

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
    let sem = Arc::new(Semaphore::new(CONCURRENT_DOWNLOAD_LIMIT));

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
    let timer = Instant::now();
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
    println!(
        "All folder files are abled to be converted into TushareInnerData ---> {:?}",
        timer.elapsed()
    );
    Ok(results)
}

// tokio和rayon的对比
// | 特性/用途    | Rayon（同步并行）                  | Tokio（异步并发）
// | ---------   | ---------------------            | ---------------------
// | 编程模型     | 多线程并行                         | 单线程事件循环 + 异步任务调度
// | 适合什么     | CPU 密集型：压缩、加密、JSON解析等    | IO 密集型：网络请求、文件读写、数据库等
// | 异步         | 不支持异步函数                     | 原生异步
// | 用于文件/网络 | 不合适                            | 非常合适

pub async fn ws_root_signin_local_sdb(
    port: u16,
    user: &str,
    pass: &str,
) -> Result<Surreal<SdbClient>, Box<dyn std::error::Error + Send + Sync>> {
    let sdb: Surreal<SdbClient> = Surreal::new::<Ws>(format!("127.0.0.1:{}", port)).await?;
    sdb.signin(Root {
        username: user,
        password: pass,
    })
    .await?;
    println!("Connected into surrealdb with ws");
    Ok(sdb)
}

// 0"ts_code", 1"trade_date",
// 2"open", 3"high", 4"low", 5"close",
// 6"pre_close", 7"change", 8"pct_chg", 9"vol", 10"amount"

pub async fn use_ns_db_record_tushareinner(
    sdb: &Surreal<SdbClient>,
    namespace: &str,
    database: &str,
    data: Vec<TushareInnerData>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    sdb.use_ns(namespace).use_db(database).await?;

    let t = std::time::Instant::now();
    let mut one_tushare_inner_data_sdbql = String::new();
    for i in data {
        for j in i.items {
            let one_line_data = format!(
                "INSERT INTO {}{}{} {} code: {}{}{}, date: {}, open: {}dec, high: {}dec, low: {}dec, close: {}dec, pre_close: {}dec, change: {}dec, chg_percent: {}dec, vol: <decimal>{}dec, amount:{}dec {}",
                r#"`"#,
                j.1.parse::<u32>().unwrap(), // table name
                r#"`"#,
                "{",
                r#"""#,
                j.0, // code
                r#"""#,
                j.1.parse::<u32>().unwrap(),                      // date
                Decimal::from_f64(j.2).unwrap(),                  // open
                Decimal::from_f64(j.3).unwrap(),                  // high
                Decimal::from_f64(j.4).unwrap(),                  // low
                Decimal::from_f64(j.5).unwrap(),                  // close
                Decimal::from_f64(j.6.unwrap_or(0f64)).unwrap(),  // pre_close
                Decimal::from_f64(j.7.unwrap_or(0f64)).unwrap(),  // change
                Decimal::from_f64(j.8.unwrap_or(0f64)).unwrap(),  // chg_percent
                Decimal::from_f64(j.9).unwrap(),                  // vol
                Decimal::from_f64(j.10.unwrap_or(0f64)).unwrap(), // amount
                "};",
            );
            one_tushare_inner_data_sdbql.push_str(&one_line_data);
        }
    }
    dbg!(t.elapsed());
    sdb.query(&one_tushare_inner_data_sdbql).await?;
    dbg!(t.elapsed());
    Ok(())
}

// #[rustfmt::skip]
// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct SdbStockStruct { code: String, date: u32, open: Decimal, high: Decimal, low: Decimal, close: Decimal, pre_close: Decimal, change: Decimal, chg_percent: Decimal, vol: Decimal, amount: Decimal }

// // #[rustfmt::skip]
// impl
//     From<(
//         String,
//         String,
//         f64,
//         f64,
//         f64,
//         f64,
//         Option<f64>,
//         Option<f64>,
//         Option<f64>,
//         f64,
//         Option<f64>,
//     )> for SdbStockStruct
// {
//     // "ts_code","trade_date","open","high","low","close","pre_close","change","pct_chg","vol","amount"
//     fn from(
//         i: (
//             String,
//             String,
//             f64,
//             f64,
//             f64,
//             f64,
//             Option<f64>,
//             Option<f64>,
//             Option<f64>,
//             f64,
//             Option<f64>,
//         ),
//     ) -> Self {
//         let date_u32 = i.1.parse::<u32>().unwrap();
//         SdbStockStruct {
//             code: i.0,
//             date: date_u32,
//             open: Decimal::from_f64(i.2).unwrap(),
//             high: Decimal::from_f64(i.3).unwrap(),
//             low: Decimal::from_f64(i.4).unwrap(),
//             close: Decimal::from_f64(i.5).unwrap(),
//             pre_close: Decimal::from_f64(i.6.unwrap_or(0f64)).unwrap(),
//             change: Decimal::from_f64(i.7.unwrap_or(0f64)).unwrap(),
//             chg_percent: Decimal::from_f64(i.8.unwrap_or(0f64)).unwrap(),
//             vol: Decimal::from_f64(i.9).unwrap(),
//             amount: Decimal::from_f64(i.10.unwrap_or(0f64)).unwrap(),
//         }
//     }
// }

// #[rustfmt::skip]
// #[derive(Debug, Clone, Serialize, Deserialize)]
// struct Record { id: surrealdb::RecordId }
