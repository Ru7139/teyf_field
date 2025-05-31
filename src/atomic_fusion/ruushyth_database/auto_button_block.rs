#[tokio::test]
async fn launch_bomb() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let year = 2001;
    let folder = "src/atomic_fusion/ruushyth_database/workshop/raw_stock_file/";

    // for i in 2001..=year {
    //     let download_folder = format!("{}/{}", folder, i);
    //     super::solution_0::http_fetch_tushare_year_dayk_use_ru_token(i, &download_folder).await?;
    // }

    for i in 2000..=year {
        let download_folder = format!("{}/{}", folder, i);
        super::solution_0::deserialize_folder_tushare_file_to_vec(&download_folder).await?;
    }

    Ok(())
}
