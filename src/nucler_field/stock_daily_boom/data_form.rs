use serde::{Deserialize, Serialize};

// tushare文件的反序列化
#[derive(Debug, Deserialize, Serialize)]
pub struct TushareJson {
    pub request_id: String,
    pub code: i32,
    pub data: TushareData,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TushareData {
    pub fields: Vec<String>,
    pub items: Vec<(String, String, f64, f64, f64, f64, f64, f64, f64, f64)>,
}

// 额外信息
#[allow(unused)]
#[derive(Debug, Serialize, Deserialize)]
pub struct ExtraInfo {
    pub extra_request_id: String,
    pub extra_response_code: i32,
    pub extra_fields_info: Vec<String>,
}

// 重新构建数据结构
#[derive(Debug, Serialize, Deserialize)]
pub struct StockDayK {
    pub basic: StBasic, // 交易所，交易所代码，股票代码
    pub date: StDate,   // 年，月，日，星
    pub daily: StDaily, // 开收高低，振幅，振幅率，成交量，成交额
}

impl Clone for StockDayK {
    fn clone(&self) -> Self {
        Self {
            basic: self.basic.clone(),
            date: self.date.clone(),
            daily: self.daily.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StBasic {
    pub exchange_code: ExcCode, // only SH, SZ, BJ
    pub exchange_name: ExcName, // only Shanghai, Shenzhen, Beijing
    pub stock_code: String,     // 可能出现000001的情况
}

impl Clone for StBasic {
    fn clone(&self) -> Self {
        Self {
            exchange_code: self.exchange_code.clone(),
            exchange_name: self.exchange_name.clone(),
            stock_code: self.stock_code.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ExcCode {
    SH,
    SZ,
    BJ,
}

impl Clone for ExcCode {
    fn clone(&self) -> Self {
        match self {
            ExcCode::SH => ExcCode::SH,
            ExcCode::SZ => ExcCode::SZ,
            ExcCode::BJ => ExcCode::BJ,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ExcName {
    Shanghai,
    Shenzhen,
    Beijing,
}

impl Clone for ExcName {
    fn clone(&self) -> Self {
        match self {
            ExcName::Shanghai => ExcName::Shanghai,
            ExcName::Shenzhen => ExcName::Shenzhen,
            ExcName::Beijing => ExcName::Beijing,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StDate {
    pub year: u32,
    pub month: u32,
    pub day: u32,
    pub weekday: StWeekday, // 仅限周一到周五
}

impl Clone for StDate {
    fn clone(&self) -> Self {
        Self {
            year: self.year,
            month: self.month,
            day: self.day,
            weekday: self.weekday.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum StWeekday {
    Monday = 1,
    Tuesday = 2,
    Wednesday = 3,
    Thursday = 4,
    Friday = 5,
}

impl Clone for StWeekday {
    fn clone(&self) -> Self {
        match self {
            StWeekday::Monday => StWeekday::Monday,
            StWeekday::Tuesday => StWeekday::Tuesday,
            StWeekday::Wednesday => StWeekday::Wednesday,
            StWeekday::Thursday => StWeekday::Thursday,
            StWeekday::Friday => StWeekday::Friday,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StDaily {
    pub open: f64,
    pub close: f64,
    pub high: f64,
    pub low: f64,
    pub price_change: f64,
    pub price_percent: f64,
    pub volume: f64,
    pub amount: f64,
}

impl Clone for StDaily {
    fn clone(&self) -> Self {
        Self {
            open: self.open,
            close: self.close,
            high: self.high,
            low: self.low,
            price_change: self.price_change,
            price_percent: self.price_percent,
            volume: self.volume,
            amount: self.amount,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Record {
    pub id: surrealdb::RecordId,
    // pub stock: StockDayK,
}
