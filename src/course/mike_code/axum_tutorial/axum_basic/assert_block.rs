use reqwest::StatusCode;

use super::struct_def::WebStateResponse;

pub async fn server_basic_check(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let rsps_main = client.get(format!("{}/xindex", url)).send().await?;
    let rsps_user = client.get(format!("{}/user/xprofile", url)).send().await?;
    let rsps_bag = client
        .get(format!("{}/display_full_state_bag", url))
        .send()
        .await?;

    assert_eq!(rsps_main.text().await?, "TEST: Hello, world");
    assert_eq!(rsps_user.text().await?, "TEST: Here is user profile");
    assert_eq!(
        rsps_bag.text().await?,
        serde_json::to_string(&WebStateResponse::new_moon_ship_response())?
    );

    Ok(())
}

// ---- ---- ---- ---- ---- zero assert defiended ---- ---- ---- ---- ----

#[rustfmt::skip]
pub async fn server_function_assert_all_check(url: &str, client: reqwest::Client) -> Result<(), Box<dyn std::error::Error>> {
    random_destination_try(url, client).await?;
    Ok(())
}

#[rustfmt::skip]
async fn random_destination_try(url: &str, client: reqwest::Client) -> Result<(), Box<dyn std::error::Error>> {
    let rsps = client
        .get(format!("{}/change_destination_random", url))
        .send()
        .await?;
    assert_eq!(rsps.status(), StatusCode::ACCEPTED);
    dbg!(rsps.text().await.unwrap());
    Ok(())
}

// ---- ---- ---- ---- ---- else assert defiended ---- ---- ---- ---- ----
