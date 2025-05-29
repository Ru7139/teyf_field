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

// use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
// use std::path::{Path, PathBuf};

// let exec_timestamp = std::time::Instant::now();
// let count_lines_closure = |file_path: &Path| -> std::io::Result<usize> {
//     Ok(std::fs::read_to_string(file_path)?.lines().count())
// };

// let rs_file_path_vec: Vec<PathBuf> =
//     walkdir::WalkDir::new(Path::new(file!()).parent().unwrap())
//         .into_iter()
//         .filter_map(|e| e.ok())
//         .filter(|e| e.file_type().is_file())
//         .map(|e| e.into_path())
//         .filter(|path| path.extension().map_or(false, |ext| ext == "rs"))
//         .collect();
// dbg!(rs_file_path_vec.len());

// let total_lines: usize = rs_file_path_vec
//     .par_iter()
//     .map(|path| count_lines_closure(path).unwrap_or(0))
//     .sum();
// dbg!(total_lines);
// dbg!(exec_timestamp.elapsed());
