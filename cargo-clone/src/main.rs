// Copyright 2015 Jan Likar.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use cargo::core::SourceId;
use cargo::util::{into_url::IntoUrl, Config};

use clap::{Arg, ArgAction};

type Result<T> = std::result::Result<T, anyhow::Error>;

fn main() {
    let app = clap::Command::new("cargo clone")
        .version(version())
        .bin_name("cargo clone")
        // A hack to make calling cargo-clone directly work.
        .arg(Arg::new("dummy")
            .hide(true)
            .required(true)
            .value_parser(["clone"]))
        .arg(
            Arg::new("color")
                .long("color")
                .value_name("COLORING")
                .help("Coloring: auto, always, never.")
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .help("Use verbose output.")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("quiet")
                .short('q')
                .help("Print less output to stdout.")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("registry")
                .long("registry")
                .value_name("REGISTRY")
                .help("A registry name from Cargo config to clone the specified crate from.")
                .conflicts_with("index"),
        )
        .arg(
            Arg::new("index")
                .long("index")
                .value_name("URL")
                .help("Registry index to install from.")
                .conflicts_with("registry"),
        )
        .arg(
            Arg::new("local-registry")
                .long("local-registry")
                .value_name("PATH")
                .help("A local registry path to clone the specified crate from.")
                .conflicts_with("registry")
                .conflicts_with("index"),
        )
        .arg(
            Arg::new("git")
                .long("git")
                .help("Clone from a repository specified in package's metadata.")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("crate")
                .help("The crates to be downloaded. Versions may also be specified and are matched exactly by default.\nExamples: 'cargo-clone@1.0.0' 'cargo-clone@~1.0.0'.")
                .required(true)
                .action(ArgAction::Append)
        )
        .arg(Arg::new("directory").help("The destination directory. If it ends in a slash, crates will be placed into its subdirectories.").last(true));

    let matches = app.get_matches();
    let mut config = Config::default().expect("Unable to get config.");

    if let Err(e) = execute(matches, &mut config) {
        config.shell().error(e).unwrap();
        std::process::exit(101);
    }
}

fn version() -> &'static str {
    let ver = format!(
        "{}.{}.{}{}",
        option_env!("CARGO_PKG_VERSION_MAJOR").unwrap_or("X"),
        option_env!("CARGO_PKG_VERSION_MINOR").unwrap_or("X"),
        option_env!("CARGO_PKG_VERSION_PATCH").unwrap_or("X"),
        option_env!("CARGO_PKG_VERSION_PRE").unwrap_or("")
    );
    Box::leak(ver.into_boxed_str())
}

pub fn execute(matches: clap::ArgMatches, config: &mut Config) -> Result<Option<()>> {
    let verbose = u32::from(matches.get_one::<bool>("verbose").is_some());

    let color: Option<&str> = matches.get_one::<String>("color").map(|s| s.as_str());
    config.configure(
        verbose,
        matches.get_one::<bool>("quiet").is_some(),
        color,
        false,
        false,
        false,
        &None,
        &[],
        &[],
    )?;

    let source_id = if let Some(registry) = matches.get_one::<String>("registry") {
        SourceId::alt_registry(config, registry)?
    } else if let Some(index) = matches.get_one::<String>("index") {
        SourceId::for_registry(&index.into_url()?)?
    } else if let Some(path) = matches.get_one::<String>("local-registry") {
        SourceId::for_local_registry(&config.cwd().join(path))?
    } else {
        SourceId::crates_io(config)?
    };

    let directory = matches.get_one::<String>("directory").map(|d| d.as_str());
    let use_git = matches.get_one::<bool>("git").is_some();

    let crates = matches
        .get_many::<String>("crate")
        .unwrap()
        .map(|c| c.as_str())
        .map(cargo_clone_core::parse_name_and_version)
        .collect::<Result<Vec<cargo_clone_core::Crate>>>()?;

    let opts = cargo_clone_core::CloneOpts::new(&crates, &source_id, directory, use_git);

    cargo_clone_core::clone(&opts, config)?;

    Ok(None)
}
