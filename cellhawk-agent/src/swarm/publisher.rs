use super::Hazard;
use redis::AsyncCommands;

pub async fn publish_hazard(
    client: &redis::Client,
    hazard: &Hazard,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut con = client.get_multiplexed_async_connection().await?;
    let json_msg = serde_json::to_string(hazard)?;
    let _: () = con.publish("danger_grid", json_msg).await?;
    Ok(())
}
