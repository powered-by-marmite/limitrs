use crate::test_server::TestServer;
use client::{CountRequest, Direction};
use tokio_tungstenite::tungstenite;
use futures::{SinkExt, StreamExt};


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

#[tokio::test]
async fn websocket_reads_counts() {
    let test_server = TestServer::spawn_server();
    
    let (mut socket, _response) = tokio_tungstenite::connect_async(
        format!("ws://{}:{}/ws/count", test_server.address, test_server.port))
      .await
      .unwrap();

    let msg = match socket.next().await.unwrap().unwrap() {
        tungstenite::Message::Ping(v) => v,
        _other => panic!("unexpected message"),
    };
    assert_eq!(msg, vec![1, 2, 3]);

    test_server.post_update(&CountRequest { direction: Direction::Increment }).await;
    test_server.assert_count_value(1).await;

    assert!(socket.send(tungstenite::Message::text("foo")).await.is_ok());

    let msg = match socket.next().await.unwrap().unwrap() {
        tungstenite::Message::Text(msg) => msg,
        _other => panic!("unexpected message"),
    };
    
    assert_eq!(msg, r#"{"count":1}"#)
}
