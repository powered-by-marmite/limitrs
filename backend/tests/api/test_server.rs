use backend::startup::run;

use std::net::{Ipv4Addr, SocketAddr, TcpListener};
use client::{CountRequest, Direction};

pub struct TestServer {
    pub address: String,
    pub port: u16,
    pub client: reqwest::Client,
}

impl TestServer {
    pub fn spawn_server() -> TestServer {
        // bind to an OS-assigned port on localhost
        let sock_addr = SocketAddr::from((Ipv4Addr::LOCALHOST, 0));
        let listener = TcpListener::bind(sock_addr).expect("failed to bind to socket");
        // retrieve the port for this socket
        let port = listener.local_addr().unwrap().port();

        // start up the server in a new green thread
        tokio::spawn(async move {
            run(listener, "".into()).await;
        });

        TestServer {
            address: Ipv4Addr::LOCALHOST.to_string(),
            port,
            client: reqwest::Client::new(),
        }
    }
    
    pub async fn assert_count_value(&self, expected: i32) {
        let response = self.client
        .get(&format!(
            "http://{}:{}/api/count",
            self.address, self.port,
        ))
        .send()
        .await
        .expect("Failed to send GET request");

    assert!(response.status().is_success());
    assert_eq!(format!(r#"{{"count":{}}}"#, expected), response.text().await.expect("GET failed"));
    }
    
    pub async fn post_update(&self, message: &CountRequest) {
        let response = self.client
        .post(&format!(
            "http://{}:{}/api/count",
            self.address, self.port,
        ))
        .json(message)
        .send()
        .await
        .expect("POST failed");

    assert!(response.status().is_success());
    }
}
