use std::process::{Child, Command};

pub struct CommandLines {
    port: u16,
    command: Command,
    child: Option<Child>,
}
impl CommandLines {
    pub fn new(port_u16: u16) -> CommandLines {
        CommandLines {
            port: port_u16,
            command: Command::new(""),
            child: None,
        }
    }
    pub fn db_start(&mut self) {
        self.command = Command::new("surreal");
        self.command
            .arg("start")
            .arg("--bind")
            .arg(&format!("127.0.0.1:{}", self.port))
            .arg("--user")
            .arg("ruut")
            .arg("--pass")
            .arg("ruut")
            .arg("file://src/course/mike_code/surrealdb_tutorial/surrealdb_basic/ruut_basic_db")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        self.child = Some(self.command.spawn().unwrap());
    }
    pub fn display_child_and_command(&self) {
        dbg!(&self.port);
        dbg!(&self.command);
        dbg!(&self.child);
    }
    pub fn kill_child(&mut self) {
        if let Some(mut child) = self.child.take() {
            child.kill().unwrap();
        };

        dbg!("command line offline");
    }
}
