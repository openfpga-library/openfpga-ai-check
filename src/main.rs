pub mod checks;
pub mod library;
pub mod output;
pub mod repo;
use std::{collections::HashMap, io::Write};

use checks::{CheckResult, all_checks};
use clap::Parser;
use octocrab::Octocrab;

use crate::{
    library::{Core, Repository, request_library_info},
    output::CoreResultsOutputFile,
    repo::RepoContext,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// cores to check in `author.name` form, will check all if none are provided
    #[arg(long, short)]
    cores: Vec<String>,

    /// output folder, will output a json file for each processed core in `author.name.json`
    #[arg(short, long, default_value = "reports")]
    output_folder: String,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args = Args::parse();
    let cores = request_library_info().await?;
    let mut cores: Vec<Core> = cores
        .into_iter()
        .filter(|c| {
            if args.cores.len() == 0 {
                true
            } else {
                args.cores.contains(&c.id)
            }
        })
        .collect();

    let current_dir = std::env::current_dir()?;
    let output_folder_path = current_dir.join(&args.output_folder);

    if !output_folder_path.exists() {
        std::fs::create_dir(&output_folder_path)?;
    }

    cores.sort_by_cached_key(|core| {
        let file_path = output_folder_path.join(format!("{}.json", &core.id));
        std::fs::File::open(&file_path)
            .ok()
            .and_then(|file| {
                let parsed: Result<CoreResultsOutputFile, _> = serde_json::from_reader(file);
                parsed.ok()
            })
            .map(|output| output.last_run)
    });

    for core in cores {
        println!("Core: {}", &core.id);

        match core.repository {
            Repository {
                platform,
                owner,
                name,
            } if platform == "github" => {
                println!("Found GitHub repo: {}/{}", owner, name);
                let repo_context = RepoContext {
                    owner: owner,
                    repo: name,
                    client: Octocrab::default(),
                };

                let mut results: HashMap<String, Vec<CheckResult>> = HashMap::new();

                for check in all_checks() {
                    results.insert(check.name().into(), check.run(&repo_context).await?);
                }

                let output_file_path = output_folder_path.join(format!("{}.json", &core.id));
                let mut file = std::fs::File::create(output_file_path)?;

                let mut output = CoreResultsOutputFile {
                    last_run: chrono::Utc::now(),
                    results: results.clone(),
                    overall_score: 0.0,
                };

                output.calculate_overall_score();

                let j = serde_json::to_string_pretty(&output)?;
                file.write(&j.into_bytes())?;
            }
            _ => {
                println!("Skipping non-GitHub platform");
            }
        }
    }

    Ok(())
}
