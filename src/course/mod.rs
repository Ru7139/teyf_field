#![cfg(test)]
pub mod mike_code;

mod count_lines_of_all_rs_file_below {
    use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
    use std::path::{Path, PathBuf};

    #[test]
    fn here_is_counting() -> Result<(), Box<dyn std::error::Error>> {
        let exec_timestamp = std::time::Instant::now();
        let count_lines_closure = |file_path: &Path| -> std::io::Result<usize> {
            Ok(std::fs::read_to_string(file_path)?.lines().count())
        };

        let rs_file_path_vec: Vec<PathBuf> =
            walkdir::WalkDir::new(Path::new(file!()).parent().unwrap())
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .map(|e| e.into_path())
                .filter(|path| path.extension().map_or(false, |ext| ext == "rs"))
                .collect();
        dbg!(rs_file_path_vec.len());

        let total_lines: usize = rs_file_path_vec
            .par_iter()
            .map(|path| count_lines_closure(path).unwrap_or(0))
            .sum();
        let not_by_myself_lines = 20597usize;
        dbg!(total_lines - not_by_myself_lines);
        dbg!(exec_timestamp.elapsed());
        Ok(())
    }
}
