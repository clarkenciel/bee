use anyhow::Context;
use clap::Parser;
use sqlx::Connection;
use tokio::io::AsyncBufReadExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let opts = Opts::parse();

    let file = tokio::fs::File::open(&opts.words_file)
        .await
        .with_context(|| anyhow::anyhow!("Failed to open file {}", opts.words_file.display()))?;
    let mut connection = sqlx::PgConnection::connect(&opts.database_url)
        .await
        .with_context(|| anyhow::anyhow!("Failed to connect to database {}", opts.database_url))?;

    let total_bytes = file.metadata().await.unwrap().len() as usize;
    let mut processed_bytes = 0;

    let mut reader = tokio::io::BufReader::new(file);
    let mut batch = Vec::with_capacity(opts.batch_size);
    let mut line = String::new();
    while let Ok(count) = reader.read_line(&mut line).await && count != 0 {
        processed_bytes += count;
        if line.len() < 4 {
            continue;
        }

        if line.trim().chars().any(|c| !c.is_ascii_alphabetic()) {
            continue;
        }

        batch.push(line.trim().to_ascii_lowercase());

        if batch.len() == opts.batch_size {
            upsert_words(&mut connection, &batch[..]).await?;
            batch.clear();
            println!("Processing: {}%", ((processed_bytes as f32 / total_bytes as f32) * 100.0) as u32);
        }
        line.clear();
    }

    println!("Done");
    Ok(())
}

/// Script to build a word database from a file containing a newline-delimited list of words.
/// This script _will_ defensively remove any word that trivially fails the checks of the
/// Spelling bee game:
///   1. >= 4 letters
///   2. all "latin" ([a-zA-Z]) characters
///
/// Words will be downcased in the the produced database.
#[derive(Debug, clap::Parser)]
struct Opts {
    /// Filepath of file containing word list from which to build words database.
    /// The file should be a newline-delimited list of words with one word per line.
    #[arg(short, long)]
    words_file: std::path::PathBuf,

    /// URL that can be used to connect to target database using SQLX.
    /// See the SQLX documentation on the DATABASE_URL environment variable for more details.
    #[arg(short, long)]
    database_url: String,

    /// Batch size of the insert batches
    #[arg(short, long, default_value_t = 1000)]
    batch_size: usize,
}

async fn upsert_words(conn: &mut sqlx::PgConnection, words: &[String]) -> anyhow::Result<()> {
    let mut builder = sqlx::QueryBuilder::new("insert into words (word, letter_mask, length) ");
    builder.push_values(words, |mut b, word| {
        let mask = words::bitmask(&word);
        let length = word.len();
        b.push_bind(word).push_bind(mask).push_bind(length as i32);
    });
    builder.push("on conflict do nothing");

    builder
        .build()
        .execute(conn)
        .await
        .with_context(|| anyhow::anyhow!("Failed to upsert word batch"))
        .map(|_| ())
}
