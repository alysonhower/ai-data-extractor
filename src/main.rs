use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Serialize)]
struct RequestBody {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Deserialize)]
struct ResponseBody {
    response: String,
}

#[derive(Error, Debug)]
enum AppError {
    #[error("Failed to read file: {0}")]
    ReadFileError(#[from] std::io::Error),
    #[error("Failed to extract text from PDF")]
    PdfExtractError,
    #[error("Failed to send request: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("Failed to parse response body")]
    ResponseParseError,
}

type Result<T> = std::result::Result<T, AppError>;

/// Simple program to find data
#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// PDF path
    #[arg(short, long)]
    path: PathBuf,

    /// Data to search
    #[arg(short, long)]
    data: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let client = reqwest::Client::new();
    let url = "http://localhost:11434/api/generate";
    let bytes = std::fs::read(args.path).map_err(AppError::ReadFileError)?;
    let document =
        pdf_extract::extract_text_from_mem(&bytes).map_err(|_| AppError::PdfExtractError)?;
    let data = args.data;

    let request_body = RequestBody {
        model: "llama3-gradient:8b-instruct-1048k-q6_K".to_string(),
        prompt: format!("You will be provided with a large amount of text, followed by a specific query to search for within that text. Your task is to carefully search through the document and return only the exact information requested, without any additional content or explanation.

Here is the document to search through:
<document>
{}
</document>

Here is the specific information to search for:
<query>
{}
</query>

Please carefully read through the entire document, searching for any text that exactly matches the query provided. 

If you find text that is an exact match to the query:
Return ONLY that matching text. Do not include the query, any labels, or any other content before or after the matching text.

If after thoroughly searching the document you do not find any text that exactly matches the query:
Return only the following text, with no other content: Desculpe, não encontrei a informação específica no documento

Begin your response immediately, without any explanation or additional information. Return only the exact matching text if found, or only the 'not found' message otherwise.", document, data),
        stream: false,
    };

    let response = client.post(url).json(&request_body).send().await?;
    let response_body: ResponseBody = response
        .json()
        .await
        .map_err(|_| AppError::ResponseParseError)?;

    println!("{}", response_body.response);

    Ok(())
}
