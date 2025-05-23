// 输入两个日期，返回有所有日期的vec
pub fn date_vec_generator(bgn_date: &str, end_date: &str) -> Result<Vec<String>, String> {
    use chrono::{Duration, NaiveDate};

    let x = NaiveDate::parse_from_str(bgn_date, "%Y%m%d").expect("error format with begin date");
    let y = NaiveDate::parse_from_str(end_date, "%Y%m%d").expect("error format with ended date");

    match &x.partial_cmp(&y).unwrap() {
        std::cmp::Ordering::Greater => Err("start after end".to_string()),
        std::cmp::Ordering::Equal => Ok(vec![bgn_date.to_string()]),
        std::cmp::Ordering::Less => {
            let answer = (0..=(y - x).num_days())
                .map(|i| (x + Duration::days(i)).format("%Y%m%d").to_string())
                .collect();
            Ok(answer)
        }
    }
}

pub fn tushare_json_body(
    api: &str,
    token: &str,
    params: serde_json::Value,
    fields: &str,
) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "api_name": api,
        "token": token,
        "params": params,
        "fields": fields
    }))
}

pub async fn send_json_get_data(
    url: &str,
    json_body: &serde_json::Value,
) -> Result<reqwest::Response, String> {
    let client = reqwest::Client::new();
    let response = client
        .post(url)
        .json(json_body)
        .send()
        .await
        .expect("Failed to send request");
    match response.status().is_success() {
        true => Ok(response),
        false => Err(format!("status code: {}", response.status())),
    }
}

pub async fn convert_response_to_string(response: reqwest::Response) -> Result<String, String> {
    Ok(response.text().await.expect("Unable to read response"))
}

pub fn write_response_into_txt(response: String, path: &str) -> Result<(), String> {
    use std::io::Write;
    let mut file = std::fs::File::create(path).expect("Unable to create file");
    file.write_all(response.as_bytes())
        .expect("Unable to write file");
    Ok(())
}

pub fn one_day_params_json(date: &String) -> serde_json::Value {
    serde_json::json! ({
        "start_date": date,
        "end_date": date,
    })
}

pub fn what_the_day_in_week(date: &str) -> Result<chrono::Weekday, Box<dyn std::error::Error>> {
    use chrono::Datelike;
    chrono::NaiveDate::parse_from_str(date, "%Y%m%d")
        .map_err(|error| error.into())
        .map(|the_day: chrono::NaiveDate| the_day.weekday())
}
