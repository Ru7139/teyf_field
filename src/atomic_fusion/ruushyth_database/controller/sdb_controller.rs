use std::process::{Child, Command};

use serde::{Deserialize, Serialize};

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
    items: Vec<(String, u16, f64, f64, f64, f64, f64, f64, f64, f64)>,
}

async fn convert_json_to_schema_vec<'a>(
    file_path: &str,
) -> Result<Vec<ChinaStockDayK>, Box<dyn std::error::Error>> {
    let file_data = std::fs::read_to_string(file_path).expect("Unable to open the file");
    let file_data_deseril: TushareMarketOneDayJson =
        serde_json::from_str(&file_data).expect("Unable to deserilize");
    let mut result: Vec<ChinaStockDayK> = Vec::with_capacity(file_data_deseril.data.items.len());
    let result: Vec<ChinaStockDayK> = file_data_deseril
        .data
        .items
        .into_iter()
        .map(Into::into)
        .collect();
    Ok(result)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ChinaStockDayK {
    code: String,
    date: u16,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    change: f64,
    chg_percent: f64,
    vol: f64,
    amount: f64,
}
impl From<(String, u16, f64, f64, f64, f64, f64, f64, f64, f64)> for ChinaStockDayK {
    fn from(i: (String, u16, f64, f64, f64, f64, f64, f64, f64, f64)) -> Self {
        ChinaStockDayK::new_with_params(i.0, i.1, i.2, i.3, i.4, i.5, i.6, i.7, i.8, i.9)
    }
}

impl ChinaStockDayK {
    fn new_with_params(
        code: String,
        date: u16,
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
