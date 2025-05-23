use std::process::{Child, Command};

pub struct SdbController {
    port: u16,
    command: Command,
    child: Option<Child>,
}
impl SdbController {
    pub fn new(port: u16) -> SdbController {
        SdbController {
            port,
            command: Command::new(""),
            child: None,
        }
    }
    pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.command = Command::new("surreal");
        self.command
            .arg("start")
            .arg("--bind")
            .arg(&format!("127.0.0.1:{}", self.port))
            .arg("--user")
            .arg("ruut_stock")
            .arg("--pass")
            .arg("ruut_stock")
            .arg("file://src/nuclear_field/stock_daily_boom/stock_sdb")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        self.child = Some(self.command.spawn().unwrap());
        Ok(())
    }
    pub fn cmd_offline(&mut self) {
        if let Some(mut child) = self.child.take() {
            child.kill().unwrap();
        };
        println!("command offline");
    }
    pub fn display_child_and_command(&self) {
        dbg!(&self.port);
        dbg!(&self.command);
        // dbg!(&self.child);
    }
}
