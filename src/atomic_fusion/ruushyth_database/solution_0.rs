pub async fn http_fetch_tushare_year_dayk_use_ru_token(
    year: i32,
    folder_path: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fetch_timer = std::time::Instant::now();

    if !std::path::Path::new(folder_path).exists() {
        std::fs::create_dir_all(folder_path)?;
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

    dbg!(fetch_timer);
    Ok(())
}
