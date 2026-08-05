use super::Hazard;
use futures_util::StreamExt;
use redis::{AsyncCommands, Value};
use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn run_subscriber(
    client: &redis::Client,
    costmap: Arc<Mutex<Vec<Hazard>>>,
) -> Result<(), redis::RedisError> {
    #[allow(deprecated)]
    let mut con = client.get_async_connection().await?.into_pubsub();
    con.subscribe("danger_grid").await?;

    let mut stream = con.on_message();

    while let Some(msg) = stream.next().await {
        let payload: String = msg.get_payload()?;
        if let Ok(hazard) = serde_json::from_str::<Hazard>(&payload) {
            let mut map = costmap.lock().await;
            map.push(hazard);
        }
    }

    Ok(())
}

pub async fn query_hazards_nearby(
    client: &redis::Client,
    lon: f64,
    lat: f64,
) -> Result<Vec<String>, redis::RedisError> {
    let mut con = client.get_multiplexed_async_connection().await?;
    // Real usage would involve GEOADD from publisher and GEOSEARCH here.
    // For now, we simulate the GEOSEARCH command wrapper via redis-rs
    // GEOSEARCH key FROMLONLAT lon lat BYRADIUS 200 m

    let result: Vec<String> = redis::cmd("GEOSEARCH")
        .arg("hazards_geo")
        .arg("FROMLONLAT")
        .arg(lon)
        .arg(lat)
        .arg("BYRADIUS")
        .arg(200)
        .arg("m")
        .query_async(&mut con)
        .await?;

    Ok(result)
}
