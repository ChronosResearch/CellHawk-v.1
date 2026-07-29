use log::{error, info};
use redis::{AsyncCommands, Client};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Debug)]
pub struct ThreatMetadata {
    pub timestamp: u64,
    pub level: f64,
    pub threat_type: String,
    pub reporter: String,
}

pub struct DangerGrid {
    client: Client,
    grid_key: String,
    expiry_sec: u64,
}

impl DangerGrid {
    pub fn new(redis_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let client = Client::open(redis_url)?;
        Ok(Self {
            client,
            grid_key: "cellhawk:danger_grid".to_string(),
            expiry_sec: 300,
        })
    }

    pub async fn report_threat(
        &self,
        drone_id: &str,
        lon: f64,
        lat: f64,
        level: f64,
        threat_type: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut con = self.client.get_async_connection().await?;

        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let threat_id = format!("threat:{}:{}", drone_id, now);

        let meta = ThreatMetadata {
            timestamp: now,
            level,
            threat_type: threat_type.to_string(),
            reporter: drone_id.to_string(),
        };
        let meta_json = serde_json::to_string(&meta)?;

        // Atomically add to GEO index and set metadata
        redis::pipe()
            .atomic()
            .cmd("GEOADD")
            .arg(&self.grid_key)
            .arg(lon)
            .arg(lat)
            .arg(&threat_id)
            .ignore()
            .cmd("SETEX")
            .arg(&threat_id)
            .arg(self.expiry_sec)
            .arg(meta_json)
            .ignore()
            .query_async(&mut con)
            .await?;

        info!(
            "Reported {} at {}, {} (Level {})",
            threat_type, lat, lon, level
        );
        Ok(())
    }

    pub async fn get_threats_in_radius(
        &self,
        lon: f64,
        lat: f64,
        radius_m: f64,
    ) -> Result<Vec<ThreatMetadata>, Box<dyn std::error::Error>> {
        let mut con = self.client.get_async_connection().await?;

        // GEOSEARCH key FROMLONLAT lon lat BYRADIUS radius m
        let result: Vec<String> = redis::cmd("GEORADIUS")
            .arg(&self.grid_key)
            .arg(lon)
            .arg(lat)
            .arg(radius_m)
            .arg("m")
            .query_async(&mut con)
            .await?;

        let mut threats = Vec::new();

        if result.is_empty() {
            return Ok(threats);
        }

        // Fetch metadata
        let metadata_results: Vec<Option<String>> = redis::cmd("MGET")
            .arg(&result)
            .query_async(&mut con)
            .await?;

        let mut expired_keys = Vec::new();

        for (threat_id, meta_opt) in result.iter().zip(metadata_results.iter()) {
            match meta_opt {
                Some(meta_str) => {
                    if let Ok(meta) = serde_json::from_str::<ThreatMetadata>(meta_str) {
                        threats.push(meta);
                    } else {
                        error!("Corrupted metadata for {}", threat_id);
                    }
                }
                None => {
                    expired_keys.push(threat_id.clone());
                }
            }
        }

        if !expired_keys.is_empty() {
            redis::cmd("ZREM")
                .arg(&self.grid_key)
                .arg(&expired_keys)
                .query_async(&mut con)
                .await?;
        }

        Ok(threats)
    }
}
