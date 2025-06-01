#[tokio::test]
async fn launch_bomb() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let year = 2000;
    let folder = "src/atomic_fusion/ruushyth_database/workshop/raw_stock_file/";

    // for i in 2000..=year {
    //     let download_folder = format!("{}/{}", folder, i);
    //     super::solution_0::http_fetch_tushare_year_dayk_use_ru_token(i, &download_folder).await?;
    // }

    let sdb = super::solution_0::ws_root_signin_local_sdb(65535, "nuut", "nuut").await?;

    for i in 2000..=year {
        let download_folder = format!("{}/{}", folder, i);
        let data =
            super::solution_0::deserialize_folder_tushare_file_to_vec(&download_folder).await?;
        let k = super::solution_0::SdbStockStruct::from(data[0].items[0].clone());
        let q = serde_json::to_string(&k).unwrap();
        dbg!(q);
        // super::solution_0::use_ns_db_record_tushareinner(&sdb, "ANS", "ADB", data).await?;
    }

    Ok(())
}
