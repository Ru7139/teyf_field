pub mod project {
    use super::super::func_tool_chain::*;

    pub async fn main_bit() -> Result<(), Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        let tushare_url = "http://api.tushare.pro";
        let api_name = "daily";
        let token_ru = "e1c23bbb77f2cc2ae0169d5f6da2b5b0df3b685763dad71085559c5a";
        let fields = "ts_code, trade_date ,open, high, low, ,close, change, pct_chg, vol, amount";
        let date_vec = date_vec_generator("20240101", "20241231")?; // 500 a/min

        // 执行从tushare获取信息
        dbg!(start_time.elapsed());
        let mut count: u32 = 0;
        let exe_for_start = std::time::Instant::now();
        for (index, datec) in date_vec.iter().enumerate() {
            // 日志打印的闭包
            let console_log_print = |counter: &mut u32| {
                if index == 0 || index % 25 == 24 || index == date_vec.len() - 1 {
                    println!(
                        "{:6.2}% ---> [{:w$}] in [{}] ---> count: [{:w$}] ---> time: {:?}",
                        ((index + 1) as f64 / date_vec.len() as f64) * 100f64,
                        index + 1,
                        date_vec.len(),
                        counter,
                        exe_for_start.elapsed(),
                        w = date_vec.len().to_string().len(),
                    )
                }
            };

            // 判断周末，周期地打印日志信息
            let day_in_week = what_the_day_in_week(datec)?;
            if day_in_week == chrono::Weekday::Sat || day_in_week == chrono::Weekday::Sun {
                console_log_print(&mut count);
                continue;
            }
            let params = one_day_params_json(&datec);
            // src/nucler_field/stock_daily_boom/download_file
            let path = format!(
                "src/nucler_field/stock_daily_boom/download_file/response_{}_Week[{}]",
                &datec, day_in_week
            );

            // 构建json，发送，转换，缓存
            let json_body = tushare_json_body(api_name, token_ru, params, fields)?;
            let response = send_json_get_data(tushare_url, &json_body).await?;
            let response_text = convert_response_to_string(response).await?;
            write_response_into_txt(response_text, path.as_str())?;

            // 进度更新并打印
            count += 1;
            console_log_print(&mut count);
        }

        dbg!(start_time.elapsed());
        Ok(())
    }
}
