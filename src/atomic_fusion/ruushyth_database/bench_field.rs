#[tokio::test]
#[ignore]
async fn sdb_test() -> Result<(), Box<dyn std::error::Error>> {
    use super::controller::sdb_controller::SdbController;
    use surrealdb::engine::remote::ws::Ws;

    let port: u16 = 60001;
    let mut test_db_ctrl = SdbController::new_with_params(
        port,
        "nuut_stock",
        "nuut_stock",
        "src/atomic_fusion/ruushyth_database/workshop/stock_fusion_db/stock_database_trunk",
    );

    test_db_ctrl.start_sdb()?;
    test_db_ctrl.display_pid();

    std::thread::sleep(std::time::Duration::from_millis(1000));

    let _sdb = surrealdb::Surreal::new::<Ws>(format!("127.0.0.1:{}", port)).await?;

    // std::thread::sleep(std::time::Duration::from_millis(500));

    test_db_ctrl.cmd_shutdown()?;

    Ok(())
}

const TUSHARE_URL: &str = "http://api.tushare.pro";
const DAILY_API: &str = "daily";
const TOKEN_RU: &str = "e1c23bbb77f2cc2ae0169d5f6da2b5b0df3b685763dad71085559c5a";
const NORMAL_FIELDS: &str =
    "ts_code, trade_date ,open, high, low, ,close, change, pct_chg, vol, amount";

#[tokio::test]
#[ignore]
async fn get_year_data_test() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let download_folder = "src/atomic_fusion/ruushyth_database/workshop/raw_stock_file/2020";
    super::controller::tushare_controller::download_tushare_data_by_day(
        super::controller::tushare_controller::get_year_days_vec(2020).unwrap(),
        TUSHARE_URL,
        DAILY_API,
        TOKEN_RU,
        NORMAL_FIELDS,
        download_folder, // 若全覆盖 14 = 3.35s
        14,              // 根据对方服务器可接受的最大压力调整 14 = 3s
    )
    .await?;
    Ok(())
}

#[tokio::test]
#[ignore]
async fn get_year_data_mix_test() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let year = 2019;
    let folder = format!(
        "src/atomic_fusion/ruushyth_database/workshop/raw_stock_file/{}",
        year
    );
    super::controller::mix_tools::download_every_stock_year_dayk_data(year, &folder, 4).await?;
    Ok(())
}

#[test]
fn convert_chinadayk_test() {
    let c_file_path = "/Users/chenzhi/Desktop/Rust/teyf_field/src/atomic_fusion/ruushyth_database/workshop/raw_stock_file/2024/rsps_20240103_[3]";
    let vec = super::controller::sdb_controller::convert_json_to_schema_vec(c_file_path);
    println!("{}", vec.len())
}
