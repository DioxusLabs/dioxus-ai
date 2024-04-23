mod generate;

mod pretty_print;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    generate::generate().await;
}
