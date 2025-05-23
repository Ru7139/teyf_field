#[tokio::test]
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

    std::thread::sleep(std::time::Duration::from_millis(500));

    test_db_ctrl.cmd_shutdown()?;

    Ok(())
}
