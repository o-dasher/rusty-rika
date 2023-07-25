use dotenvy::dotenv;
use tokio::try_join;

#[tokio::main]
pub async fn main() {
    dotenv().ok();

    let result_work = try_join!(rika_bancho::run(), rika_poise::run());

    if let Err(e) = result_work {
        println!("{e:?}")
    }
}
