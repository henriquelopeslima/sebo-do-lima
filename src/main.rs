use sebo_do_lima::config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    sebo_do_lima::run(Config::from_env()?).await
}
