use backend::startup::run;
use clap::Parser;
use std::net::{IpAddr, Ipv6Addr, SocketAddr, TcpListener};
use std::str::FromStr;

const LOGGING_VARIABLE: &str = "RUST_LOG";

// Setup the command line interface with clap.
#[derive(Parser, Debug)]
#[clap(name = "server", about = "A server for our wasm project!")]
struct Opt {
    /// set the log level
    #[clap(short = 'l', long = "log-level", default_value = "debug")]
    log_level: String,

    /// set the listen addr
    #[clap(short = 'a', long = "addr", default_value = "::1")]
    addr: String,

    /// set the listen port
    #[clap(short = 'p', long = "port", default_value = "8080")]
    port: u16,

    /// set the directory where static files are to be found
    #[clap(long = "static-dir", default_value = "./dist")]
    static_dir: String,
}

#[tokio::main]
async fn main() {
    let opt = Opt::parse();

    // set up logging
    match std::env::var(LOGGING_VARIABLE) {
        Ok(_) => (),
        Err(_) => std::env::set_var(
            LOGGING_VARIABLE,
            format!("{},hyper=info,mio=info", opt.log_level),
        ),
    }
    // log to the console
    tracing_subscriber::fmt::init();

    let sock_addr = SocketAddr::from((
        IpAddr::from_str(opt.addr.as_str()).unwrap_or(IpAddr::V6(Ipv6Addr::LOCALHOST)),
        opt.port,
    ));
    let listener = TcpListener::bind(sock_addr).expect("failed to bind to socket");

    log::info!("listening on http://{}", sock_addr);
    run(listener, opt.static_dir).await
}
