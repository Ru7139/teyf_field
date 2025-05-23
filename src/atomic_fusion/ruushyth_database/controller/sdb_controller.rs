use std::process::{Child, Command};

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
