pub async fn db_start(port: u16) {
    let mut command = std::process::Command::new("surreal");
    command
        .arg("start")
        .arg("--bind")
        .arg(&format!("127.0.0.1:{}", port))
        .arg("--user")
        .arg("ruut")
        .arg("--pass")
        .arg("ruut")
        .arg("file://src/course/mike_code/surrealdb_tutorial/surrealdb_basic/ruut_basic_db")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    let mut child = command.spawn().unwrap();
    dbg!(&mut child);
}
