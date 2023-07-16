#[derive(Default, sqlx::FromRow)]
pub struct OsuPerformance {
    #[sqlx(default)]
    pub id: i64,

    #[sqlx(default)]
    pub aim: f64,

    #[sqlx(default)]
    pub speed: f64,

    #[sqlx(default)]
    pub accuracy: f64,

    #[sqlx(default)]
    pub flashlight: f64,

    #[sqlx(default)]
    pub overall: f64,
}

#[derive(Default, sqlx::FromRow)]
pub struct OsuScore {
    #[sqlx(default)]
    pub id: i64,

    #[sqlx(default)]
    pub osu_user_id: i64,

    #[sqlx(default)]
    pub mods: i32,

    #[sqlx(default)]
    pub map_id: i64,

    #[sqlx(default)]
    pub mode: i16,

    #[sqlx(flatten)]
    pub performance: OsuPerformance,
}
