#![forbid(unsafe_code)]

fn main() {
    let mut args = std::env::args().skip(1);

    if matches!(args.next().as_deref(), Some("--version" | "-V")) {
        println!("agent-cli {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    println!("{}", agent_core::runtime_banner());
}
