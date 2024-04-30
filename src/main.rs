mod generate;

// mod pretty_print;
// mod ux;

// mod html;

#[tokio::main]
async fn main() {
    // tracing_subscriber::fmt::init();

    generate::generate().await;
    // ux::ux().await;
}
