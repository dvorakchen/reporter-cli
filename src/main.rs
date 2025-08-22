use std::path::Path;

use clap::{Parser, Subcommand};
use reporter_cli::{
    crawler::HotNewsCrawler, director::Director, news::NewsTitle, source::SourceName,
};
use tokio::fs;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// List hot news
    List {
        /// hot news source,
        /// allows:
        ///
        /// - pengpai
        /// - ithome
        name: String,
    },
    /// shot a short video of news
    Shot {
        #[arg(long)]
        source: String,
        #[arg(long)]
        url: String,
        #[arg(long)]
        title: String,
        #[arg(long, value_delimiter = ';')]
        pics: Vec<String>,
        #[arg(long, value_delimiter = ';')]
        videos: Vec<String>,
        #[arg(long)]
        author: String,
        #[arg(long)]
        tts_url: String,
        #[arg(long)]
        openai_url: String,
        #[arg(long)]
        openai_key: String,
        #[arg(long)]
        model: String,
        #[arg(long)]
        output: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), &'static str> {
    let args = Args::parse();

    match args.command {
        Commands::List { name } => {
            get_hot_nows_list(&name).await;
        }
        Commands::Shot {
            source,
            url,
            title,
            pics,
            videos,
            author,
            tts_url,
            openai_url,
            openai_key,
            model,
            output
        } => {
            take_a_shot(
                NewsTitle {
                    source,
                    url,
                    title,
                    pics,
                    videos,
                },
                &author,
                &tts_url,
                &openai_url,
                &openai_key,
                &model,
                &output
            )
            .await;
        }
    }

    Ok(())
}

async fn get_hot_nows_list(source: &SourceName) {
    let crawl = HotNewsCrawler::new();
    let list = crawl.get(source).await.expect("get hot news list failed");

    let json = serde_json::to_string(&list).expect("serde to json failed");
    println!("{}", json);
}

async fn take_a_shot(
    title: NewsTitle,
    author: &str,
    tts_url: &str,
    openai_url: &str,
    openai_key: &str,
    model: &str,
    output: &str,
) {
    let director = Director::new(
        author.to_string(),
        tts_url.to_string(),
        openai_url.to_string(),
        openai_key.to_string(),
        model.to_string(),
    );
    let output_path = Path::new(output);
    if let Some(parent) = output_path.parent() && !parent.exists() {
        _ = fs::create_dir_all(parent).await;
    }

    let result = director.shot_single(title, output_path).await.expect("take shot fail");

    let json = serde_json::to_string(&result).expect("serde to json failed");
    println!("{}", json);
}
