const TUSHARE_URL: &str = "http://api.tushare.pro";
const DAILY_API: &str = "daily";
const TOKEN_RU: &str = "e1c23bbb77f2cc2ae0169d5f6da2b5b0df3b685763dad71085559c5a";
const NORMAL_FIELDS: &str =
    "ts_code, trade_date ,open, high, low, ,close, change, pct_chg, vol, amount";
const NORMAL_YEAR_DAYS: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
const LEAP_YEAR_DAYS: [u32; 12] = [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
const YEAR_DAYS: [[u32; 12]; 2] = [NORMAL_YEAR_DAYS, LEAP_YEAR_DAYS]; // false, ture

pub async fn get_year_data(
    year: u32,
    year_folder_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let func_exec_timer = std::time::Instant::now();
    let mut date_vec: Vec<u32> = Vec::with_capacity(366);

    let months_days = YEAR_DAYS[(year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)) as usize];

    for (month_id, days) in months_days.into_iter().enumerate() {
        for the_day in 1..=days {
            date_vec.push(year * 10000 + (month_id as u32 + 1) * 100 + the_day);
        }
    }

    dbg!(date_vec);

    Ok(())
}
