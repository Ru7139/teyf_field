use super::rulib;

#[tokio::test]
async fn the_learning_of_rust() -> bool {
    let hello_vec: Vec<u8> = vec![72, 101, 108, 108, 111];
    let world_vec: Vec<u8> = vec![87, 111, 114, 108, 100];
    let begin_always_string = String::from_utf8(
        Vec::new()
            .into_iter()
            .chain(hello_vec)
            .chain([44, 32])
            .chain(world_vec)
            .chain([33])
            .collect::<Vec<u8>>(),
    )
    .unwrap();
    let begin_always_base64 = "SGVsbG8sIFdvcmxkIQ==";
    assert_eq!(
        begin_always_base64,
        rulib::ru_encode_base64(begin_always_string.as_str())
            .unwrap()
            .as_str()
    );

    true
}
