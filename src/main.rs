mod regex_pd;
mod filter;
mod identifier;
mod options;
mod format;

use crate::filter::{create_filter, parse_rarity, Filter};
use crate::format::{get_format, output};
use crate::identifier::{identify, Match};
use crate::options::{Options, OutputFormat};
use crate::regex_pd::load_regex_pattern_data;
use anyhow::{Context, Result};
use clap::{Arg, Command};
use clap_complete::aot::{generate, Generator, Shell};
use human_panic::setup_panic;
use std::{io, process};

const HELP_TEMPLATE_FORMAT: &str = "\
{before-help}{name} {version}
{about-with-newline}
{author-with-newline}
{usage-heading} {usage}

{all-args}{after-help}
";

const JSON_DATA: &str = include_str!("../data/regex.json");

fn cli() -> Command {
    Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .help_template(HELP_TEMPLATE_FORMAT)
        .arg(
            Arg::new("input")
                .help("Input to identify. Input can be text, a file or directory.")
                .required(false),
        )
        .arg(
            Arg::new("tags")
                .short('t')
                .long("tags")
                .help("Show available tags and exit.")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("rarity")
                .short('r')
                .long("rarity")
                .default_value("0.1:1")
                .help("Filter by rarity, range of 0:1."),
        )
        .arg(
            Arg::new("include")
                .short('i')
                .long("include")
                .help("Only show matches with these tags."),
        )
        .arg(
            Arg::new("exclude")
                .short('e')
                .long("exclude")
                .help("Exclude matches with these tags."),
        )
        .arg(
            Arg::new("only_text")
                .short('o')
                .long("only-text")
                .help("Do not scan files or folders.")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("disable-borderless")
                .short('d')
                .long("disable-borderless")
                .help("Disable borderless mode.")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("format")
                .long("format")
                .help("Output format.")
                .value_parser(["default", "json", "pretty"]),
        )
        .arg(
            Arg::new("reverse")
                .long("reverse")
                .help("Reverse the sorting order.")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("key")
                .long("key")
                .value_parser(["name", "rarity", "matched", "none"])
                .default_value("none")
                .default_missing_value("none")
                .help("Filter by key name."),
        )
        .arg(
            Arg::new("generate")
                .long("generate")
                .help("Generate a shell completion file for the specified shell")
                .value_parser(["bash", "zsh", "fish", "powershell", "elvish"]),
        )
}

fn main() {
    setup_panic!();

    clap_complete::CompleteEnv::with_factory(cli)
        .completer("exhaustive")
        .complete();

    let matches = cli().get_matches();
    if let Some(generator) = matches.get_one::<Shell>("generate") {
        let mut cmd = cli();
        eprintln!("Generating completion file for {generator}...");
        print_completions(*generator, &mut cmd);
        return;
    }

    if matches.get_flag("tags") {
        print_tags().unwrap();
        process::exit(0);
    }

    let regex_data = load_regex_pattern_data(JSON_DATA).unwrap();

    let rarity = matches
        .get_one::<String>("rarity")
        .map(|s| parse_rarity(s))
        .transpose()
        .context("Invalid rarity range format. Expected 'min:max'").unwrap();

    let filter: Filter = create_filter(
        rarity,
        !matches.get_flag("disable-borderless"),
        matches.get_one::<String>("include"),
        matches.get_one::<String>("exclude"),
    );

    let mut options: Options = Options {
        only_text: matches.get_flag("only_text"),
        format: OutputFormat::DEFAULT,
    };

    options.format = get_format(&matches.get_one::<String>("format"));

    let input = matches.get_one::<String>("input").cloned();
    if input.is_none() {
        println!("{} (Version: {})", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        println!("\n{}", env!("CARGO_PKG_DESCRIPTION"));
        println!("Made by {}", env!("CARGO_PKG_AUTHORS"));
        println!("For more information see {}", env!("CARGO_PKG_HOMEPAGE"));
        eprintln!("\nRun '--help' for usage.");
        process::exit(1);
    }

    // Determine if the input is text or a file/directory path
    if let Some(input) = matches.get_one::<String>("input") {
        let mut matches: Vec<Match> = Vec::new();
        identify(input, regex_data, &mut matches, &filter, &options).unwrap();
        output(&matches, &options)
    } else {
        eprintln!("Input as text or file/directory path expected. Run '--help' for usage.");
        process::exit(1);
    }
}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}

fn print_tags() -> Result<()> {
    println!("Available Tags:");
    // TODO: Code to retrieve and print available tags goes here
    Ok(())
}




