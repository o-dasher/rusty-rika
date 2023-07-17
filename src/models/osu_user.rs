use sqlx::{mysql::MySqlArguments, query::Query, MySql};

pub struct OsuUser {}

impl OsuUser {
    pub fn try_create(osu_id: &u32) -> Query<MySql, MySqlArguments> {
        sqlx::query!(
            "
            INSERT IGNORE INTO osu_user (id)
            VALUES (?)
            ",
            osu_id
        )
    }
}

