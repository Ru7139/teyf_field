mod course;
mod metextbook;
mod nucler_field;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // stock
    {
        // nucler_field::stock_daily_boom::main_block::project::main_fetch_data().await?;
        // nucler_field::stock_daily_boom::main_block::project::main_deconstruct_local_data_try().await?;
    }

    // option
    {
        // use nucler_field::stock_daily_boom::option_bsmm::OptionStruct;
        // let mut option_example = OptionStruct::default();
        // option_example.test_et_display();
    }
    Ok(())
}
