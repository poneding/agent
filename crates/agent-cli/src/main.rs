#![forbid(unsafe_code)]

fn main() {
    match agent_cli::parse_cli_invocation(std::env::args().skip(1)) {
        Ok(invocation) => {
            let output = agent_cli::render_cli_invocation(invocation);
            print!("{}", output.stdout);
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    }
}
