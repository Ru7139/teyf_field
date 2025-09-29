use rand::prelude::Distribution;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct WebStateSharedBag {
    rocket: Arc<Mutex<CosmosRocket>>,
    destination: Arc<Mutex<TargetLocationXYZ>>,
    information: Arc<Mutex<UniversalGoldDisk>>,
}
impl WebStateSharedBag {
    pub fn new_moon_ship() -> WebStateSharedBag {
        WebStateSharedBag {
            rocket: Arc::new(Mutex::new(CosmosRocket {
                ship_type: CosmosRocketType::RoamerShip,
                ship_code: u32::MAX.isqrt(),
            })),
            destination: Arc::new(Mutex::new(TargetLocationXYZ {
                x: 150f64,
                y: 30f64,
                z: -45f64,
            })),
            information: Arc::new(Mutex::new(UniversalGoldDisk {
                music: "BWV 1006".into(),
                writter: "Bach".into(),
            })),
        }
    }
    pub async fn into_web_state_response(
        self,
    ) -> Result<WebStateResponse, Box<dyn std::error::Error>> {
        Ok(WebStateResponse {
            rocket: self.rocket.lock().await.clone(),
            destination: self.destination.lock().await.clone(),
            information: self.information.lock().await.clone(),
        })
    }
    pub async fn change_to_random_location(&self, range: f64) {
        self.destination
            .lock()
            .await
            .change_to_random_location(range);
    }
    pub async fn get_destination_xyz(self) -> (f64, f64, f64) {
        let destination = self.destination.lock().await.clone();
        (destination.x, destination.y, destination.z)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebStateResponse {
    rocket: CosmosRocket,
    destination: TargetLocationXYZ,
    information: UniversalGoldDisk,
}
impl WebStateResponse {
    pub fn new_moon_ship_response() -> WebStateResponse {
        WebStateResponse {
            rocket: CosmosRocket {
                ship_type: CosmosRocketType::RoamerShip,
                ship_code: u32::MAX.isqrt(),
            },
            destination: TargetLocationXYZ {
                x: 150f64,
                y: 30f64,
                z: -45f64,
            },
            information: UniversalGoldDisk {
                music: "BWV 1006".into(),
                writter: "Bach".into(),
            },
        }
    }
}

#[rustfmt::skip]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CosmosRocket { ship_type: CosmosRocketType, ship_code: u32 }

#[rustfmt::skip]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TargetLocationXYZ { x: f64, y: f64, z: f64 }
impl TargetLocationXYZ {
    pub fn change_to_random_location(&mut self, range: f64) {
        let mut rng = rand::thread_rng();
        self.x = rand::distributions::Uniform::from(-range..range).sample(&mut rng);
        self.y = rand::distributions::Uniform::from(-range..range).sample(&mut rng);
        self.z = rand::distributions::Uniform::from(-range..range).sample(&mut rng);
    }
}

#[rustfmt::skip]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UniversalGoldDisk { music: String, writter: String }

// ---- ---- ---- ---- ---- struct defiended ---- ---- ---- ---- ----

#[rustfmt::skip]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CosmosRocketType { MotherShip, BattleShip, RoamerShip }

// ---- ---- ---- ---- ---- enum defiended ---- ---- ---- ---- ----
