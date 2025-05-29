#[tokio::test]
async fn launch_bomb() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let year = 2021;
    let folder = "src/atomic_fusion/ruushyth_database/workshop/raw_stock_file/";
    let download_folder = format!("{}/{}", folder, year);

    super::solution_0::http_fetch_tushare_year_dayk_use_ru_token(year, &download_folder).await?;

    Ok(())
}
