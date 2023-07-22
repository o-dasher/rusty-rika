use sqlx::types::time::OffsetDateTime;

#[derive(sqlx::FromRow)]
pub struct OsuPerformance {
    pub id: u64,
    pub aim: f32,
    pub speed: f32,
    pub flashlight: f32,
    pub accuracy: f32,
    pub overall: f32,
}

#[derive(sqlx::FromRow)]
pub struct TaikoPerformance {
    pub id: u64,
    pub accuracy: f32,
    pub difficulty: f32,
    pub overall: f32,
}

#[derive(sqlx::FromRow)]
pub struct ManiaPerformance {
    pub id: u64,
    pub difficulty: f32,
    pub overall: f32,
}

#[derive(sqlx::FromRow)]
pub struct OsuScore {
    pub id: u64,
    pub osu_user_id: u32,
    pub osu_score_id: u64,
    pub mods: u32,
    pub map_id: u32,
    pub created_at: OffsetDateTime,
    pub mode: i16,
}
