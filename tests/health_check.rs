use std::net::TcpListener;

// `tokio::test`is the testing equivalent of `tokio::main`.
// It also spares you from having to specify the `#[test]` attribute.
//
// You can inspect what code gets generated using
// `cargo expand --test health_check` (<-name of the test file)
#[tokio::test]
async fn health_check_works() {
    // Arrange
    let address = spawn_app();
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(&format!("{}/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

fn spawn_app() -> String {
    let listner = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");

    // retrieve the port assigned by the OS
    let port = listner.local_addr().unwrap().port();

    let server = newsletter::run(listner).expect("Failed to bind address");

    // launch the server as a background task
    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}
