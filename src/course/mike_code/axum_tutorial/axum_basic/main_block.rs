mod project {
    use super::super::assert_block::{server_basic_check, server_function_assert_all_check};
    use super::super::major_router::basic_server;
    use super::super::struct_def::WebStateSharedBag;

    #[tokio::test]
    #[ignore]
    async fn main() -> Result<(), Box<dyn std::error::Error>> {
        let web_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let web_url = format!("http://{}", web_listener.local_addr()?);

        let web_server = basic_server(WebStateSharedBag::new_moon_ship());

        tokio::spawn(async move { axum::serve(web_listener, web_server).await.unwrap() });
        server_basic_check(&web_url).await?;

        let client = reqwest::Client::new();
        server_function_assert_all_check(&web_url, client).await?;

        Ok(())
    }
}
