mod error;

use error::Error;
use getopts::Options;
use rusoto_core::Region;
use rusoto_sns::{PublishInput, Sns, SnsClient};
use serde::Deserialize;
use std::{env, str::FromStr};

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} <message>", program);
    print!("{}", opts.usage(&brief));
}

#[derive(Deserialize)]
struct SnsCredentials {
    pub sns_topic: String,
    pub aws_access_key_id: String,
    pub aws_secret_access_key: String,
    pub aws_region: String,
}


fn main() -> Result<(), Error> {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt(
        "r",
        "region",
        "AWS region of the SNS (defaults to us-east-2)",
        "",
    );

    opts.optopt("t", "topic-arn", "Topic ARN", "");
    opts.optopt("s", "subject", "Subject", "");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };

    let topic_arn = matches.opt_str("t").unwrap_or("arn:aws:sns:us-east-2:250463611689:email-me".to_string());
    let subject = matches.opt_str("s");
    let region = matches.opt_str("r").unwrap_or("us-east-2".to_string());

    let message = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        print_usage(&program, opts);
        return Ok(());
    };

    // Send the message to our SNS topic
    let client = SnsClient::new_with(
        rusoto_core::HttpClient::new().expect("Failed to create HTTP client"),
        rusoto_core::credential::ChainProvider::new(),
        Region::from_str(&region)?);

    println!("Subject: {}", subject.clone().unwrap_or("".to_string()));
    println!("Message: {}", message);

    // Unfortunately we need the entire futures/tokio jungle attached to our
    // gorilla and banana in order to use rusoto.
    let _response = tokio::runtime::Runtime::new().expect("Failed to setup tokio runtime").block_on(
    client.publish(
        PublishInput {
            message,
            message_attributes: None,
            message_deduplication_id: None,
            message_group_id: None,
            message_structure: None,
            phone_number: None,
            subject,
            target_arn: None,
            topic_arn: Some(topic_arn),
        }
    ))?;

    Ok(())
}
