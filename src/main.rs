mod course;
mod nucler_field;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    nucler_field::stock_daily_boom::main_block::project::main_bit().await?;
    Ok(())
}
