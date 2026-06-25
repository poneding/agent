#![forbid(unsafe_code)]

fn main() {
    match agent_cli::parse_cli_command(std::env::args().skip(1)) {
        Ok(command) => {
            let output = agent_cli::render_cli_command(command);
            print!("{}", output.stdout);
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    }
}
