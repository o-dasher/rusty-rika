#[derive(sqlx::FromRow)]
pub struct RikaUser {
    pub id: i64,

    pub osu_id: Option<i64>,
    pub discord_id: Option<String>,
}
