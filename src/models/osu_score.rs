use sqlx::types::time::OffsetDateTime;

pub struct OsuPerformance {
    pub id: u64,
    pub aim: f32,
    pub speed: f32,
    pub flashlight: f32,
    pub accuracy: f32,
    pub overall: f32,
}

pub struct TaikoPerformance {
    pub id: u64,
    pub accuracy: f32,
    pub difficulty: f32,
    pub overall: f32,
}

pub struct OsuScore {
    pub id: u64,
    pub osu_user_id: u32,
    pub mods: u32,
    pub map_id: u32,
    pub created_at: OffsetDateTime,
    pub mode: i16,
}
