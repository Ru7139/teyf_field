use std::io::Write;
use std::io::{BufRead, Read}; //  reader.lines // write!

pub fn start_run_rc(syslog_path: &str, start_time: chrono::DateTime<chrono::Utc>) {
    let mut system_log = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .append(true)
        .open(&syslog_path)
        .unwrap();
    writeln!(
        system_log,
        "argc ---> [{0}]\nargv ---> {1:?}\nRUNx00_00 ---> {2}",
        std::env::args().count(),
        std::env::args(),
        &start_time
    )
    .unwrap();
    drop(system_log);
} // 建议在程序开始时运行，需要传入文件地址和当前时间两个参数,用于记录 ---> argc，argv，开始时间

pub fn endet_run_rc(syslog_path: &str, end_time: chrono::DateTime<chrono::Utc>) {
    let mut system_log = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .append(true)
        .open(&syslog_path)
        .unwrap();
    let reader = std::io::BufReader::new(&system_log);
    let mut last_run_time: Option<chrono::DateTime<chrono::Utc>> = None;
    for line in reader.lines() {
        let line = line.unwrap();
        if let Some(time_str) = line.split(" ---> ").nth(1) {
            if let Ok(time) = time_str.parse::<chrono::DateTime<chrono::Utc>>() {
                last_run_time = Some(time);
            }
        }
    }
    if let Some(last_run) = last_run_time {
        writeln!(
            system_log,
            "RUNxFF_FF ---> {0}\nCost_time ---> {1}\n\n",
            end_time,
            -(last_run - end_time)
        )
        .unwrap();
    }
    drop(system_log);
} // 建议在程序结束前运行，需要传入文件地址和当前时间两个参数,用于记录 ---> 结束时间，消耗时间

pub fn my_grep() -> Result<(), std::io::Error> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Usage:rugrep <query> <filename>",
        ));
    }
    let query = &args[1];
    let filename = &args[2];
    if query == "rugrep" {
        let contents = std::fs::read_to_string(filename)?;
        println!("With text:\n{}", contents);
        Ok(())
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Only 'rugrep' is supported as the query",
        ))
    }
} // 必须传入2个以上的参数，且第一个必须是minigrep，第二个是文件名字，该功能是在crate下搜索文件名并输出内容

pub fn ru_encode_base64(x: &str) -> Result<String, std::io::Error> {
    let mut b64_encoder =
        base64::write::EncoderStringWriter::new(&base64::engine::general_purpose::STANDARD);
    b64_encoder.write_all(x.as_bytes())?;
    let b64_string = b64_encoder.into_inner();
    Ok(b64_string)
}

pub fn ru_decode_base64(x: &str) -> Result<String, std::io::Error> {
    let mut data = std::io::Cursor::new(x);
    let mut b64_decoder =
        base64::read::DecoderReader::new(&mut data, &base64::engine::general_purpose::STANDARD);
    let mut decoded_data = Vec::new();
    b64_decoder.read_to_end(&mut decoded_data)?;
    Ok(String::from_utf8(decoded_data).unwrap())
}
