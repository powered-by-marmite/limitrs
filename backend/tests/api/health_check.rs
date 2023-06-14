use crate::test_server::TestServer;

#[tokio::test]
async fn health_check_works() {
    let test_server = TestServer::spawn_server();

    let response = test_server.client
        .get(&format!(
            "http://{}:{}/health_check",
            test_server.address, test_server.port,
        ))
        .send()
        .await
        .expect("Failed to send GET request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}
