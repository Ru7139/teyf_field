#[tokio::test]
// #[ignore]
async fn sdb_test() -> Result<(), Box<dyn std::error::Error>> {
    // use super::controller::sdb_controller::SdbController;
    use surrealdb::engine::remote::ws::Ws;

    let timer = std::time::Instant::now();

    let port: u16 = 65534;
    // let mut test_db_ctrl = SdbController::new_with_params(
    //     port,
    //     "nuut_stock",
    //     "nuut_stock",
    //     "src/atomic_fusion/ruushyth_database/workshop/stock_fusion_db/stock_database_trunk",
    // );

    // test_db_ctrl.start_sdb()?;
    // test_db_ctrl.display_pid();

    // std::thread::sleep(std::time::Duration::from_millis(1000));

    let sdba = surrealdb::Surreal::new::<Ws>(format!("127.0.0.1:{}", port)).await?;

    sdba.signin(surrealdb::opt::auth::Root {
        username: "nuut_stock",
        password: "nuut_stock",
    })
    .await?;

    let namespace = "ruushath";
    let database = "Tey2021";
    let concurrent_num = 5000;

    let dir_path = "/Users/chenzhi/Desktop/Rust/teyf_field/src/atomic_fusion/ruushyth_database/workshop/raw_stock_file/2021/";
    let ignored = [".DS_Store", "Thumbs.db"];
    let file_paths: Vec<std::path::PathBuf> = walkdir::WalkDir::new(dir_path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| {
            e.file_type().is_file() && !ignored.contains(&e.file_name().to_string_lossy().as_ref())
        })
        .map(|e| e.path().to_path_buf())
        .collect();

    use super::controller::sdb_controller::convert_json_to_schema_vec;

    for x in file_paths {
        let data_vec = convert_json_to_schema_vec(x.to_str().unwrap());

        super::controller::sdb_controller::save_dayk_to_sdb(
            &sdba,
            namespace,
            database,
            data_vec,
            concurrent_num,
        )
        .await?;
    }

    // std::thread::sleep(std::time::Duration::from_millis(500));

    // test_db_ctrl.cmd_shutdown()?;

    dbg!(timer.elapsed());
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
    let year = 2022;
    let folder = format!(
        "src/atomic_fusion/ruushyth_database/workshop/raw_stock_file/{}",
        year
    );
    super::controller::mix_tools::download_every_stock_year_dayk_data(year, &folder, 12).await?;
    Ok(())
}

#[test]
#[ignore]
fn convert_chinadayk_test() {
    let c_file_path = "/Users/chenzhi/Desktop/Rust/teyf_field/src/atomic_fusion/ruushyth_database/workshop/raw_stock_file/2024/rsps_20240101_[1]";
    let vec = super::controller::sdb_controller::convert_json_to_schema_vec(c_file_path);
    println!("{}", vec.len());
}

#[test]
#[ignore]
fn convert_one_folder_chinadayk_test() -> Result<(), Box<dyn std::error::Error>> {
    let dir_path = "/Users/chenzhi/Desktop/Rust/teyf_field/src/atomic_fusion/ruushyth_database/workshop/raw_stock_file/2021/";

    let ignored = [".DS_Store", "Thumbs.db"];
    let file_paths: Vec<std::path::PathBuf> = walkdir::WalkDir::new(dir_path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| {
            e.file_type().is_file() && !ignored.contains(&e.file_name().to_string_lossy().as_ref())
        })
        .map(|e| e.path().to_path_buf())
        .collect();
    for p in file_paths {
        // println!("{:?}", p);
        let _vec =
            super::controller::sdb_controller::convert_json_to_schema_vec(p.to_str().unwrap());
        // dbg!(vec.len());
    }

    Ok(())
}
