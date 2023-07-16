#[derive(Default, sqlx::FromRow, Clone)]
pub struct OsuPerformance {
    pub id: i64,
    pub aim: f64,
    pub speed: f64,
    pub accuracy: f64,
    pub flashlight: f64,
    pub overall: f64,
}

#[derive(Default, sqlx::FromRow)]
pub struct OsuScore {
    pub id: i64,
    pub osu_user_id: i64,
    pub mods: i32,
    pub map_id: i32,
    pub mode: i16,
}
