use std::process::{Child, Command};

use dioxus::html::u::data;
use serde::{Deserialize, Serialize};
use surrealdb::{Surreal, engine::remote::ws::Client};

pub struct SdbController {
    sdb_port: u16,
    user: String,
    pass: String,
    database_folder: String,
    command_line: Option<Command>,
    childa: Option<Child>,
}
impl SdbController {
    pub fn new_with_params(
        port: u16,
        user_name: &str,
        password: &str,
        sdb_src_path: &str,
    ) -> SdbController {
        SdbController {
            sdb_port: port,
            user: user_name.into(),
            pass: password.into(),
            database_folder: sdb_src_path.into(),
            command_line: Some(Command::new("surreal")),
            childa: None,
        }
    }
    pub fn start_sdb(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut x = self
            .command_line
            .take()
            .ok_or("Command template already used")?;

        x.arg("start")
            .arg("--bind")
            .arg(&format!("127.0.0.1:{}", self.sdb_port))
            .arg("--user")
            .arg(&self.user)
            .arg("--pass")
            .arg(&self.pass)
            .arg(&format!("file://{}", &self.database_folder))
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        self.childa = Some(x.spawn()?);
        Ok(())
    }

    pub fn display_pid(&self) {
        if self.childa.is_some() == true {
            let process_id = self.childa.as_ref().map(|child| child.id());
            if let Some(id) = process_id {
                println!("surrealdb pid = {}", id);
            }
        }
    }
    pub fn cmd_shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(mut child) = self.childa.take() {
            child.kill()?;
        }
        dbg!("command offline");
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TushareMarketOneDayJson {
    request_id: String,
    code: i32,
    data: TushareCruxData,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct TushareCruxData {
    fields: Vec<String>,
    items: Vec<(String, String, f64, f64, f64, f64, f64, f64, f64, f64)>,
}

pub fn convert_json_to_schema_vec(file_path: &str) -> Vec<ChinaStockDayK> {
    let file_data = std::fs::read_to_string(file_path).expect("Unable to open the file");
    let file_data_deseril: TushareMarketOneDayJson =
        serde_json::from_str(&file_data).expect("Unable to deserilize");
    let vec_len = file_data_deseril.data.items.len();
    if vec_len == 0 {
        return Vec::new();
    }
    let mut result: Vec<ChinaStockDayK> = Vec::with_capacity(vec_len);
    let result: Vec<ChinaStockDayK> = file_data_deseril
        .data
        .items
        .into_iter()
        .map(Into::into) // ChatGPT: 非常轻量（O(1) 内存移动/复制）
        .collect();
    result
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChinaStockDayK {
    code: String,
    date: u32,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    change: f64,
    chg_percent: f64,
    vol: f64,
    amount: f64,
}
impl From<(String, String, f64, f64, f64, f64, f64, f64, f64, f64)> for ChinaStockDayK {
    fn from(i: (String, String, f64, f64, f64, f64, f64, f64, f64, f64)) -> Self {
        let date: u32 = i.1.parse().unwrap();
        ChinaStockDayK::new_with_params(i.0, date, i.2, i.3, i.4, i.5, i.6, i.7, i.8, i.9)
    }
}

impl ChinaStockDayK {
    fn new_with_params(
        code: String,
        date: u32,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        change: f64,
        chg_percent: f64,
        vol: f64,
        amount: f64,
    ) -> ChinaStockDayK {
        ChinaStockDayK {
            code,
            date,
            open,
            high,
            low,
            close,
            change,
            chg_percent,
            vol,
            amount,
        }
    }
}

use futures::stream::{self, StreamExt};
pub async fn save_dayk_to_sdb(
    sdb: &Surreal<Client>,
    namespace: &str,
    database: &str,
    data_vec: Vec<ChinaStockDayK>,
    concurrent_num: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    sdb.use_ns(namespace).use_db(database).await?;

    stream::iter(data_vec)
        .map(|x| {
            let sdb = sdb.clone();
            async move {
                sdb.create((x.date.to_string().as_str(), &x.code)) // table name & id(main key)
                    .content(x)
                    .await
            }
        })
        .buffer_unordered(concurrent_num) // 最大并发数
        .collect::<Vec<Result<Option<Record>, surrealdb::Error>>>()
        .await
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;
    Ok(())
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Record {
    id: surrealdb::RecordId,
}
