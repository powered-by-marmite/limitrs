use crate::test_server::TestServer;
use client::{CountRequest, Direction};

#[tokio::test]
async fn increment_and_decrement_count_works() {
    let test_server = TestServer::spawn_server();

    test_server.assert_count_value(0).await;
    test_server
        .post_update(&CountRequest {
            direction: Direction::Increment,
        })
        .await;
    test_server.assert_count_value(1).await;
    test_server
        .post_update(&CountRequest {
            direction: Direction::Decrement,
        })
        .await;
    test_server.assert_count_value(0).await;
}
