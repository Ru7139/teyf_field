mod atomic_fusion;
mod course;
mod metextbook;
mod nuclear_field;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    Ok(())
}

// stock
// nucler_field::stock_daily_boom::main_block::project::main_fetch_data().await?;
// let stock_list: Vec<nuclear_field::stock_daily_boom::data_form::StockDayK> =
//     nuclear_field::stock_daily_boom::main_block::project::main_deconstruct_local_data_try()
//         .await
//         .expect("error with file");
// nuclear_field::stock_daily_boom::main_block::project::push_dayk_into_sdb_try(stock_list)
//     .await
//     .expect("error with sdb");

// option
// use nuclear_field::stock_daily_boom::option_bsmm::OptionStruct;
// let mut option_example = OptionStruct::default();
// option_example.test_et_display();
