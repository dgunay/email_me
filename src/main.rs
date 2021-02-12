mod error;

use anyhow::Result;
use body::to_bytes;
use getopts::Options;
use hyper::{
    body::{self},
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server, StatusCode,
};
use rusoto_core::Region;
use rusoto_sns::{PublishError, PublishInput, PublishResponse, Sns, SnsClient};
use serde::Deserialize;
use serde_json::json;
use std::{env, net::SocketAddr, str::FromStr};
use tokio::runtime::Runtime;

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
        // Run a server to continually service requests
        Runtime::new()
            .unwrap()
            .block_on(serve_requests(&client, &topic_arn))?;
    } else {
        // Make a single command-line invocation
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
            .block_on(publish_message(&client, message, subject, &topic_arn))?;
    }

    Ok(())
}

async fn serve_requests(client: &SnsClient, topic_arn: &String) -> Result<()> {
    // We'll bind to 127.0.0.1:3000
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    // A `Service` is needed for every connection, so this
    // creates one from our function.
    let make_svc = make_service_fn(move |_conn| {
        let client = client.clone();
        let topic_arn = topic_arn.clone();
        async {
            Ok::<_, anyhow::Error>(service_fn(move |req| {
                let client = client.clone();
                let topic_arn = topic_arn.clone();
                async move { handle_request(&client, &topic_arn, req).await }
            }))
        }
    });

    let server = Server::bind(&addr).serve(make_svc);

    // Run this server for... forever!
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }

    Ok(())
}

#[derive(Deserialize, Debug)]
struct Payload {
    subject: Option<String>,
    message: String,
}

/// Creates a 500 response from the error.
fn err_to_response(err: impl ToString) -> Result<Response<Body>> {
    Ok(Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(
            serde_json::to_string(&json!({
            "message" : err.to_string()}))
            .expect("Failed to serialize JSON")
            .into(),
        )
        .expect("Failed to construct body"))
}

/// Thin wrapper that converts errors into 500 responses.
async fn handle_request(
    client: &SnsClient,
    topic_arn: &String,
    req: Request<Body>,
) -> Result<Response<Body>> {
    service_publish_message(&client, &topic_arn, req)
        .await
        .or_else(err_to_response)
}

async fn service_publish_message(
    client: &SnsClient,
    topic_arn: &String,
    mut req: Request<Body>,
) -> Result<Response<Body>> {
    dbg!("in service");

    // Attempt to deserialize the body as json
    // TODO: there are theoretically ways of streaming this instead of allocating
    // a Vec each time, but it takes a lot of work. See the destream crate.
    // https://docs.rs/destream/0.3.0/destream/

    // We need to elegantly capture errors and turn them into 500 Responses.
    let bytes = to_bytes(req.body_mut()).await?.to_vec();
    let payload: Payload = serde_json::from_slice(&bytes)?;

    // Publish the message
    let publish_response =
        publish_message(&client, payload.message, payload.subject, topic_arn).await?;
    Ok(Response::new(
        serde_json::to_string(&publish_response)?.into(),
    ))
}

// Send the message to our SNS topic
async fn publish_message(
    client: &SnsClient,
    message: String,
    subject: Option<String>,
    topic_arn: &String,
) -> Result<PublishResponse, rusoto_core::RusotoError<PublishError>> {
    println!("Subject: {}", subject.clone().unwrap_or("".to_string()));
    println!("Message: {}", message);

    client
        .publish(PublishInput {
            message,
            message_attributes: None,
            message_deduplication_id: None,
            message_group_id: None,
            message_structure: None,
            phone_number: None,
            subject,
            target_arn: None,
            topic_arn: Some(topic_arn.clone()),
        })
        .await
}
