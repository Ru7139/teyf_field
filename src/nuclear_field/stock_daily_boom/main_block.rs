pub mod project {
    use crate::nuclear_field::stock_daily_boom::db_tool_chain;

    use super::super::data_form::*;
    use super::super::func_tool_chain::*;
    use chrono::prelude::*;
    use rayon::iter::IntoParallelRefIterator;
    use rayon::iter::ParallelIterator;
    use serde::{Deserialize, Serialize};
    use surrealdb::RecordId;
    use surrealdb::engine::remote::ws::Ws;

    pub async fn main_fetch_data() -> Result<(), Box<dyn std::error::Error>> {
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

    pub async fn main_deconstruct_local_data_try()
    -> Result<Vec<StockDayK>, Box<dyn std::error::Error>> {
        let exe_start = std::time::Instant::now();

        //反序列化，得到信息
        let file_folder = "/Users/chenzhi/Desktop/Rust/teyf_field/src/nuclear_field/stock_daily_boom/download_file";
        let file_name = "response_20240102_Week[Tue]";
        let file_abs_path = format!("{}/{}", file_folder, file_name);
        let file_data = std::fs::read_to_string(file_abs_path).expect("can not read path");
        let file_deseril_json: TushareJson = serde_json::from_str(&file_data)?;

        //处理额外信息
        #[allow(unused)]
        let extra_info = ExtraInfo {
            extra_request_id: file_deseril_json.request_id,
            extra_response_code: file_deseril_json.code,
            extra_fields_info: file_deseril_json.data.fields.clone(),
        };

        //拆分信息，将值以struct保存到vec中
        let stock_list_in_one_day: Vec<StockDayK> = file_deseril_json
            .data
            .items
            .iter()
            .map(|x| {
                let ts_code = &x.0;
                let trade_date: u32 = x.1.parse().unwrap();
                let ts_open = x.2;
                let ts_high = x.3;
                let ts_low = x.4;
                let ts_close = x.5;
                let ts_change = x.6;
                let ts_pct_chg = x.7;
                let ts_vol = x.8;
                let ts_amount = x.9;

                let ts_code_match_re = ts_code.split('.').nth(1).unwrap();

                let stock_basic = StBasic {
                    exchange_code: match ts_code_match_re {
                        "SH" => ExcCode::SH,
                        "SZ" => ExcCode::SZ,
                        "BJ" => ExcCode::BJ,
                        _ => panic!("Stock is not from SH, SZ or BJ"),
                    },
                    exchange_name: match ts_code_match_re {
                        "SH" => ExcName::Shanghai,
                        "SZ" => ExcName::Shenzhen,
                        "BJ" => ExcName::Beijing,
                        _ => panic!("Stock is not from SH, SZ or BJ"),
                    },
                    stock_code: ts_code.split('.').nth(0).unwrap().to_string(),
                };

                let stock_date = StDate {
                    year: trade_date / 10000,
                    month: (trade_date / 100) % 100,
                    day: trade_date % 100,
                    weekday: match chrono::NaiveDate::parse_from_str(
                        &trade_date.to_string(),
                        "%Y%m%d",
                    ) {
                        Ok(date) => match date.weekday() {
                            chrono::Weekday::Mon => StWeekday::Monday,
                            chrono::Weekday::Tue => StWeekday::Tuesday,
                            chrono::Weekday::Wed => StWeekday::Wednesday,
                            chrono::Weekday::Thu => StWeekday::Thursday,
                            chrono::Weekday::Fri => StWeekday::Friday,
                            _ => panic!("Saturday or Sunday should be market-closed"),
                        },
                        Err(_) => panic!("Failed to parse date"),
                    },
                };

                let stock_daily = StDaily {
                    open: ts_open,
                    close: ts_close,
                    high: ts_high,
                    low: ts_low,
                    price_change: ts_change,
                    price_percent: ts_pct_chg,
                    volume: ts_vol,
                    amount: ts_amount,
                };

                StockDayK {
                    basic: stock_basic,
                    date: stock_date,
                    daily: stock_daily,
                }
            })
            .collect();

        // dbg!(stock_list_in_one_day[0].clone());

        Ok(stock_list_in_one_day)
    }

    pub async fn push_dayk_into_sdb_try(
        stock_vec: Vec<StockDayK>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let exe_time = std::time::Instant::now();
        let port: u16 = 13344;
        let mut cmd = db_tool_chain::SdbController::new(port);
        cmd.start().expect("can not find surreal database");
        // cmd.display_child_and_command();

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        // print!("Connected sdb, timer: [{:?}]", exe_time.elapsed());
        // tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let sdb = surrealdb::Surreal::new::<Ws>(format!("127.0.0.1:{}", port)).await?;
        sdb.signin(surrealdb::opt::auth::Root {
            username: "ruut_stock",
            password: "ruut_stock",
        })
        .await?;
        sdb.use_ns("nuclear_pope").use_db("stock_day_k").await?;

        for x in stock_vec {
            let _record: Option<Record> = sdb.create("stock").content(x).await?;
        }
        println!("Done file data record, timer: [{:?}]", exe_time.elapsed());
        cmd.cmd_offline();
        Ok(())
    }
    #[derive(Debug, Serialize, Deserialize, Clone)]
    struct Record {
        id: RecordId,
    }
}
