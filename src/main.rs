use serde::Deserialize;
use std::convert::Infallible;
use warp::Filter;
use reqwest::Client;
use serde_json::json;
use tokio::signal;
use std::error::Error;

static DISCORD_ENV_KEY: &'static str = "DISCORD_WEBHOOK";
static SVC_PORT_ENV_KEY: &'static str = "PORT";

#[derive(Deserialize)]
struct GithubPayload {
    action: String,
    repository: Repository,
    sender: Sender,
}

#[derive(Deserialize)]
struct Repository {
    full_name: String,
}

#[derive(Deserialize)]
struct Sender {
    login: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let port: u16 = load_port();


    // println!("Started server on port {:?}", port);
    // Create a warp filter to handle POST requests to "/webhook"
    let routes = warp::path("webhook")
        .and(warp::post())
        .and(warp::body::json()) // Automatically deserializes JSON payload
        .and_then(handle_github_webhook);
    let (addr, server) = warp::serve(routes)
    .bind_with_graceful_shutdown(([127, 0, 0, 1], port), async move {

        signal::ctrl_c()
            .await
            .expect("failed to listen to shutdown signal");
        println!("Closing server...");
    });

    println!("Started server at {:?}", addr);

    server.await;

    Ok(())
}

// Handler for GitHub webhook
async fn handle_github_webhook(payload: GithubPayload) -> Result<impl warp::Reply, Infallible> {


    // TODO(mindflayer): Add sha256 payload integrity to validate the request is real:
    // https://docs.github.com/en/webhooks/using-webhooks/validating-webhook-deliveries#testing-the-webhook-payload-validation

    // Example logic to send a message to Discord on a push event
    if payload.action == "push" {
        let message = format!(
            "New push to repository: {}\nby user: {}",
            payload.repository.full_name,
            payload.sender.login
        );

        // Send the message to Discord
        if let Err(e) = send_to_discord(&message).await {
            eprintln!("Error sending message to Discord: {}", e);
            return Ok(warp::reply::with_status("Error", warp::http::StatusCode::INTERNAL_SERVER_ERROR));
        }

        Ok(warp::reply::with_status("Webhook received", warp::http::StatusCode::OK))
    } else {
        Ok(warp::reply::with_status("Not a push event", warp::http::StatusCode::OK))
    }
}

fn load_webhook_url() -> String {
    match std::env::var(DISCORD_ENV_KEY) {
        Ok(val) => {
            if val == "changeme" {
                panic!("Please change the default value for env var {}", DISCORD_ENV_KEY)
            }
            return val
        },
        Err(_) => panic!("Environment variable {} not set", DISCORD_ENV_KEY),
    }
}

fn load_port() -> u16 {
    match std::env::var(SVC_PORT_ENV_KEY) {
        Ok(val) => {
            match val.parse::<u16>() {
                Ok(res) => return res,
                Err(_) => panic!("{} not a valid port number, got {}", SVC_PORT_ENV_KEY, val),
            }
        },
        Err(_) => panic!("Environment variable {} not set", SVC_PORT_ENV_KEY),
    }
}

// Function to send the message to Discord via webhook
async fn send_to_discord(message: &str) -> Result<(), reqwest::Error> {
    let discord_webhook_url: String = load_webhook_url();

    // Create the payload
    let payload: serde_json::Value = json!({
        "content": message,
    });

    // Send the payload to the Discord webhook
    let client: Client = Client::new();
    client.post(discord_webhook_url)
        .json(&payload)
        .send()
        .await?
        .error_for_status()?; // This ensures we handle non-2xx status codes

    Ok(())
}
