use serde_json::json;
use std::io::Write;

const NORMAL_YEAR_DAYS: [i32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
const LEAP_YEAR_DAYS: [i32; 12] = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
const YEAR_DAYS: [[i32; 12]; 2] = [NORMAL_YEAR_DAYS, LEAP_YEAR_DAYS]; // false, ture
const WEEK_DAY_SAKAMOTO_ARRAY: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];

pub fn get_year_days_vec(year: i32) -> Result<Vec<i32>, Box<dyn std::error::Error>> {
    let days_array = YEAR_DAYS[(year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)) as usize];

    let mut date_vec: Vec<i32> = Vec::with_capacity(366);
    date_vec.extend(
        days_array
            .into_iter()
            .enumerate()
            .flat_map(|(m, d)| (1..=d).map(move |day| year * 10000 + (m as i32 + 1) * 100 + day)),
    ); // dbg!(date_vec.capacity(), date_vec.len()); // 366, 366

    Ok(date_vec)
}

pub async fn download_tushare_data_by_day(
    year_days_vec: Vec<i32>,
    url: &str,
    api: &str,
    token: &str,
    fields: &str,
    download_folder_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let timer = std::time::Instant::now();
    let vec_len = year_days_vec.len();
    let mut counter = 0u32;

    // 日志打印的闭包
    let console_log_print = |total: usize, now_id: usize, counter: u32| {
        if now_id == 0 {
            println!("Now beginning to fetch!");
        }
        if now_id == 0 || now_id % 25 == 24 || now_id == total - 1 {
            println!(
                "{:6.2}% ---> [{:w$}] in [{}] ---> count: [{:w$}] ---> time: {:?}",
                ((now_id + 1) as f64 / total as f64) * 100f64,
                now_id + 1,
                total,
                counter,
                timer.elapsed(),
                w = total.to_string().len(),
            )
        }
    };

    for (ctr, ymd) in year_days_vec.into_iter().enumerate() {
        let dt = (ymd / 10000, (ymd / 100) % 100, ymd % 100);
        let y = if dt.1 >= 3 { dt.0 } else { dt.0 - 1 };
        let ya = y + y / 4 - y / 100 + y / 400;
        let week_day_num = (ya + WEEK_DAY_SAKAMOTO_ARRAY[(dt.1 - 1) as usize] + dt.2) % 7;

        if week_day_num == 6 || week_day_num == 0 {
            console_log_print(vec_len, ctr, counter);
            continue;
        }

        let response = client
            .post(url)
            .json(&json!({
                "api_name": api,
                "token": token,
                "params": json!({ "start_date": ymd, "end_date": ymd }),
                "fields": fields
            }))
            .send()
            .await
            .expect("Failed to send request");

        match response.status().is_success() {
            false => return Err(format!("status code: {}", response.status()).into()),
            true => {
                let file_path = format!("{}/rsps_{}_[{}]", download_folder_path, ymd, week_day_num);
                let mut file = std::fs::File::create(file_path).expect("Unable to create file");
                file.write_all(response.text().await?.as_bytes())
                    .expect("Unable to write");
            }
        }
        counter += 1;
        console_log_print(vec_len, ctr, counter);
    }

    Ok(())
}
