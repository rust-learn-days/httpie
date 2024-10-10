use std::collections::HashMap;
use std::str::FromStr;

use clap::Parser;
use colored::Colorize;
use mime::Mime;
use reqwest::{header, Client, Response};

#[derive(Parser)]
#[clap(name = "httpie", version = "0.1.0", about = "A CLI HTTP client")]
#[clap(setting = clap::AppSettings::ColoredHelp)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    Get(Get),
    Post(Post),
}

#[derive(Parser)]
struct Get {
    #[clap(parse(try_from_str = parse_url))]
    url: String,
}

fn parse_url(s: &str) -> Result<String, String> {
    if s.starts_with("http://") || s.starts_with("https://") {
        Ok(s.to_string())
    } else {
        Err("URL must start with http:// or https://".to_string())
    }
}

#[derive(Parser)]
struct Post {
    #[clap(parse(try_from_str = parse_url))]
    url: String,
    #[clap(parse(try_from_str = parse_key_value))]
    body: Vec<KeyValue>,
}

#[derive(Debug)]
struct KeyValue {
    key: String,
    value: String,
}

impl FromStr for KeyValue {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('=').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err(anyhow::anyhow!(
                "Key value pair must be in the format key=value"
            ));
        }
        Ok(KeyValue {
            key: parts[0].to_string(),
            value: parts[1].to_string(),
        })
    }
}

fn parse_key_value(s: &str) -> Result<KeyValue, String> {
    match KeyValue::from_str(s) {
        Ok(kv) => Ok(kv),
        Err(e) => Err(e.to_string()),
    }
}

#[tokio::main]
#[allow(clippy::let_unit_value)]
async fn main() -> Result<(), anyhow::Error> {
    let opts: Opts = Opts::parse();
    let mut headers = header::HeaderMap::new();
    headers.insert(header::ACCEPT, header::HeaderValue::from_static("*/*"));
    headers.insert(
        header::USER_AGENT,
        header::HeaderValue::from_static("rust-client"),
    );
    env_logger::init();
    let client = Client::builder()
        .no_proxy()
        .default_headers(headers)
        .build()?;
    let result = match opts.subcmd {
        SubCommand::Get(ref args) => get(client, args).await?,
        SubCommand::Post(ref args) => post(client, args).await?,
    };
    Ok(result)
}
#[allow(clippy::needless_question_mark)]
async fn get(client: Client, args: &Get) -> Result<(), anyhow::Error> {
    let res = client.get(&args.url).send().await?;
    Ok(print_resp(res).await?)
}
#[allow(clippy::needless_question_mark)]
async fn post(client: Client, args: &Post) -> Result<(), anyhow::Error> {
    let mut body = HashMap::new();
    for kv in args.body.iter() {
        body.insert(&kv.key, &kv.value);
    }
    let res = client.post(&args.url).json(&body).send().await?;
    Ok(print_resp(res).await?)
}

fn print_status(res: &Response) {
    let status = format!("{:?} {}", res.version(), res.status()).blue();
    println!("{}", status);
}

fn print_headers(res: &Response) {
    for (name, value) in res.headers().iter() {
        println!(
            "{}: {}",
            name.to_string().green(),
            value.to_str().unwrap().cyan()
        );
    }
    println!("\n")
}

fn print_body(m: Option<Mime>, body: &String) {
    match m {
        Some(v) if v.type_() == mime::APPLICATION && v.subtype() == mime::JSON => {
            println!("{}", jsonxf::pretty_print(body).unwrap().cyan());
        }
        _ => {
            println!("{}", body.cyan());
        }
    }
}

//
async fn print_resp(res: Response) -> Result<(), anyhow::Error> {
    if res.status().is_client_error() {
        println!("Error Client Status: {:?}", res.status());
    }
    if res.status().is_server_error() {
        println!("Error Server Status: {:?}", res.status());
    }
    print_status(&res);
    print_headers(&res);
    let m = get_content_type(&res);
    match res.text().await {
        Ok(body) => print_body(m, &body),
        Err(e) => println!("Failed to read response body: {}", e),
    }
    Ok(())
}

fn get_content_type(res: &Response) -> Option<Mime> {
    res.headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse().ok())
}
