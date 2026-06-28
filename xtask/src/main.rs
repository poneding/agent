#![forbid(unsafe_code)]

use std::process::{Command, ExitStatus};

use xtask::{Task, command_plan_for, help_text, parse_args, run_package_cli, run_package_desktop};

fn main() {
    let task = match parse_args(std::env::args().skip(1)) {
        Ok(Task::Help) => {
            print!("{}", help_text());
            return;
        }
        Ok(Task::PackageCli(options)) => {
            match run_package_cli(&options) {
                Ok(manifest) => {
                    println!(
                        "packaged {} ({})",
                        manifest.artifact.name, manifest.artifact.checksum_sha256
                    );
                }
                Err(error) => {
                    eprintln!("package-cli failed: {error}");
                    std::process::exit(1);
                }
            }
            return;
        }
        Ok(Task::PackageDesktop(options)) => {
            match run_package_desktop(&options) {
                Ok(manifest) => {
                    println!(
                        "packaged desktop metadata {} ({})",
                        manifest.version,
                        manifest.channel.as_str()
                    );
                }
                Err(error) => {
                    eprintln!("package-desktop failed: {error}");
                    std::process::exit(1);
                }
            }
            return;
        }
        Ok(task) => task,
        Err(error) => {
            eprintln!("{error}\n");
            eprint!("{}", help_text());
            std::process::exit(2);
        }
    };

    for command in command_plan_for(&task) {
        let status = run_command(&command.program, &command.args).unwrap_or_else(|error| {
            eprintln!(
                "failed to run {} {}: {error}",
                command.program,
                command.args.join(" ")
            );
            std::process::exit(1);
        });

        if !status.success() {
            std::process::exit(status.code().unwrap_or(1));
        }
    }
}

fn run_command(program: &str, args: &[String]) -> std::io::Result<ExitStatus> {
    Command::new(program).args(args).status()
}
