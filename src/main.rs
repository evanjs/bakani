use clap::{crate_version, crate_name, crate_authors, crate_description};
use tracing::{error};
use bakani::get_baka_entry_from_url;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::from_default_env()).init();

    let matches = clap::Command::new(crate_name!())
        .author(crate_authors!())
        .version(crate_version!())
        .about(crate_description!())
        .arg(
            clap::Arg::new("INPUT")
                .help("The query on which to operate (search, etc.)")
                .required(true)
                .index(1)
        )
        .arg(
            clap::Arg::new("search")
                .short('s')
                .help("Search for the provided query")
        )
        .get_matches();

    let query: String = matches.value_of_t_or_exit("INPUT");
    bakani::search_and_get_baka_entry(&query).await.and_then( |r| {
        println!("Results: {}", r);
        Ok(())
    }).map_err(|e| {
        error!(error =? e, "Failed to search for title: {}", query);
    }).ok();

    Ok(())
}
