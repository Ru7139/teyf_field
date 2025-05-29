pub async fn http_fetch_tushare_year_dayk_use_ru_token(
    year: i32,
    folder_path: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use std::{
        fs::create_dir_all,
        path::Path,
        sync::{Arc, atomic::AtomicUsize},
        time::Instant,
    };

    use serde_json::json;
    use std::io::Write;

    let done_fetch_timer = Instant::now();

    if !Path::new(folder_path).exists() {
        create_dir_all(folder_path)?;
    }

    let year_days: [[i32; 12]; 2] = [
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31],
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31],
    ];

    let mut date_vec: Vec<i32> = Vec::with_capacity(366);

    date_vec.extend(
        year_days[(year % 4 == 0 && (year % 400 == 0 || year % 100 != 0)) as usize]
            .into_iter()
            .enumerate()
            .flat_map(|(m, d)| (1..=d).map(move |x: i32| year * 10000 + (m as i32 + 1) * 100 + x)),
    );

    let client = reqwest::Client::new();
    let downloaded_file_counter = Arc::new(AtomicUsize::new(0));

    type JHRst = tokio::task::JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>>;
    let mut tasks: Vec<JHRst> = Vec::with_capacity(366);
    let semaphore = Arc::new(tokio::sync::Semaphore::new(10usize));

    let sakamoto_week_day_array: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let tushare_url = "http://api.tushare.pro";
    let daily_api = "daily";
    let token_ru = "e1c23bbb77f2cc2ae0169d5f6da2b5b0df3b685763dad71085559c5a";
    // let TOKEN_FE = "7ec7fdbb1c5d4c384becfdc5bcc0df6932503ea1a858dbf02196dabb";
    let normal_fields =
        "ts_code, trade_date ,open, high, low, ,close, pre_close, change, pct_chg, vol, amount";

    for ymd in date_vec.into_iter() {
        let dt = (ymd / 10000, (ymd / 100) % 100, ymd % 100); // date
        let y = if dt.1 >= 3 { dt.0 } else { dt.0 - 1 }; // year adjust
        let ya = y + y / 4 - y / 100 + y / 400; // part formula
        let week_day_num = (ya + sakamoto_week_day_array[(dt.1 - 1) as usize] + dt.2) % 7;

        if week_day_num == 6 || week_day_num == 0 {
            continue;
        } // 如果为周六日则跳过

        let semaphore_clone = std::sync::Arc::clone(&semaphore); // 克隆所需的变量
        let client_clone = client.clone();
        let folder_path_clone = folder_path.to_string();
        let counter_clone = std::sync::Arc::clone(&downloaded_file_counter);

        let task = tokio::task::spawn(async move {
            let _permit = semaphore_clone.acquire().await.unwrap(); // 限制并发数量

            let response = client_clone
                .post(tushare_url)
                .json(&json!({
                    "api_name": daily_api,
                    "token": token_ru,
                    "params": json!({ "start_date": ymd, "end_date": ymd }),
                    "fields": normal_fields
                }))
                .send()
                .await
                .expect("Failed to send request");

            match response.status().is_success() {
                false => {
                    return Err(format!("status code: {}", response.status()).into());
                }
                true => {
                    let file_path =
                        format!("{}/rsps_{}_[{}]", folder_path_clone, ymd, week_day_num);
                    let mut file = std::fs::File::create(file_path).expect("Unable to create file");
                    file.write_all(response.text().await?.as_bytes())
                        .expect("Unable to write");
                }
            }

            counter_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

            Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
        });

        tasks.push(task);
    }

    for task in tasks {
        task.await??;
    }

    dbg!(downloaded_file_counter);
    dbg!(done_fetch_timer.elapsed());
    Ok(())
}
