mod error;

use futures::TryFutureExt;
use getopts::Options;
use hyper::{Body, Request, Response, Server, body::to_bytes, service::{make_service_fn, service_fn}};
use rusoto_core::Region;
use rusoto_sns::{PublishInput, PublishResponse, Sns, SnsClient};
use serde::Deserialize;
use tokio::runtime::Runtime;
use std::{convert::Infallible, env, net::SocketAddr, str::FromStr};
use anyhow::Result;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} <message>", program);
    print!("{}", opts.usage(&brief));
}

fn main() -> Result<()> {
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
    opts.optflag("e", "server", "Serve requests continuously");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };
    let region = matches.opt_str("r").unwrap_or("us-east-2".to_string());
    let client = SnsClient::new_with(
        rusoto_core::HttpClient::new().expect("Failed to create HTTP client"),
        rusoto_core::credential::ChainProvider::new(),
        Region::from_str(&region)?,
    );

    let topic_arn = matches
        .opt_str("t")
        .unwrap_or("arn:aws:sns:us-east-2:250463611689:email-me".to_string());

    if matches.opt_present("e") {
        Runtime::new().unwrap().block_on(serve_requests());
    } else {
        let subject = matches.opt_str("s");

        let message = if !matches.free.is_empty() {
            matches.free[0].clone()
        } else {
            print_usage(&program, opts);
            return Ok(());
        };

        // Unfortunately we need the entire futures/tokio jungle attached to our
        // gorilla and banana in order to use rusoto, even if we only need to
        // make a single blocking call.
        let _resp = tokio::runtime::Runtime::new()
            .expect("Failed to setup tokio runtime")
            .block_on(publish_message(&client, message, subject, topic_arn))?;
    }

    Ok(())
}

async fn serve_requests() -> Result<()> {
    // We'll bind to 127.0.0.1:3000
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    // A `Service` is needed for every connection, so this
    // creates one from our function.
    let make_svc = make_service_fn(|_conn| async {
        // service_fn converts our function into a `Service`
        Ok::<_, Infallible>(service_fn(service_publish_message))
    });

    let server = Server::bind(&addr).serve(make_svc);

    // Run this server for... forever!
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }

    Ok(())
}

#[derive(Deserialize)]
struct Payload {
    subject: Option<String>,
    message: String,
}

async fn service_publish_message(req: Request<Body>) -> Result<Response<Body>> {
    // Attempt to deserialize the body as json
    let payload: Payload = serde_json::from_slice(req.body())?;

    Ok(Response::new("Message sent.".into()))
}

// Send the message to our SNS topic
async fn publish_message(
    client: &SnsClient,
    message: String,
    subject: Option<String>,
    topic_arn: String,
) -> Result<PublishResponse, rusoto::error::> {
    println!("Subject: {}", subject.clone().unwrap_or("".to_string()));
    println!("Message: {}", message);

    client.publish(PublishInput {
        message,
        message_attributes: None,
        message_deduplication_id: None,
        message_group_id: None,
        message_structure: None,
        phone_number: None,
        subject,
        target_arn: None,
        topic_arn: Some(topic_arn),
    }).into().await
}
