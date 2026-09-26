use bakani::BakaClient;
use clap::{crate_authors, crate_description, crate_name, crate_version};
use tracing::error;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let matches = clap::Command::new(crate_name!())
        .author(crate_authors!())
        .version(crate_version!())
        .about(crate_description!())
        .arg(
            clap::Arg::new("INPUT")
                .help("The query on which to operate (search, etc.)")
                .required(true)
                .index(1),
        )
        .arg(
            clap::Arg::new("search")
                .short('s')
                .help("Search for the provided query"),
        )
        .get_matches();

    let query: String = matches.value_of_t_or_exit("INPUT");
    let client = BakaClient::new();
    client
        .search_and_get_baka_entry(&query)
        .await
        .and_then(|r| {
            println!("{}", r);
            Ok(())
        })
        .map_err(|e| {
            error!(error =? e, "Failed to search for title: {}", query);
        })
        .ok();

    Ok(())
}
