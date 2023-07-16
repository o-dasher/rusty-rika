use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub struct BeatmapCache {
    pub client: reqwest::Client,
    pub cache: Arc<Mutex<HashMap<u32, Arc<[u8]>>>>,
}

impl BeatmapCache {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get_beatmap_file(&self, beatmap_id: u32) -> Result<Arc<[u8]>, anyhow::Error> {
        {
            if let Some(cached) = self.cache.lock().unwrap().get(&beatmap_id) {
                return Ok(cached.clone());
            };
        }

        let response = self
            .client
            .get(&format!("https://osu.ppy.sh/osu/{beatmap_id}"))
            .send()
            .await?;

        let map_bytes: Arc<[u8]> = response
            .bytes()
            .await
            .and_then(|bytes| Ok(Vec::<u8>::from(bytes).into()))?;

        {
            self.cache
                .lock()
                .unwrap()
                .insert(beatmap_id, map_bytes.clone());
        }

        Ok(map_bytes)
    }
}
