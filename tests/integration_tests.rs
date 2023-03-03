#[cfg(test)]
mod tests {
    use async_listener::file_system;
    use std::time::Duration;
    use tempfile::tempdir;
    use tokio::sync::mpsc::channel;

    #[tokio::test]
    async fn test_async_watch() {
        // Create a temporary directory and file to watch
        let temp_dir = tempdir().expect("failed to create temp dir");
        let temp_file = temp_dir.path().join("test.txt");
        let temp_file_clone = temp_file.clone();

        std::fs::write(&temp_file, "hello world").expect("failed to write to temp file");

        // Create a channel for communication between the test and the watcher task
        let (tx, mut rx) = channel(1);

        // Spawn a task to listen for file system events
        let handle = tokio::spawn(async move {
            let mut count = 0;
            let result = file_system::async_watch(temp_file).await;
            if let Err(e) = &result {
                println!("async_watch error: {:?}", e);
            }
            assert!(result.is_ok());
            count += 1;
            println!("count: {}", count);

            // Wait for the test to signal that it's time to stop
            rx.recv().await.expect("channel closed");

            // Check that at least 3 events were received
            assert!(count >= 3);
        });

        // Wait for the event listener to start up
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Modify the file to trigger a file system event
        std::fs::write(&temp_file_clone, "goodbye world").expect("failed to write to temp file");

        // Modify the file to trigger another file system event
        std::fs::write(&temp_file_clone, "hello again").expect("failed to write to temp file");

        // Modify the file to trigger a third file system event
        std::fs::write(&temp_file_clone, "goodbye again").expect("failed to write to temp file");

        // Wait for the events to be processed
        tokio::time::sleep(Duration::from_millis(1000)).await;

        // Signal to the watcher task that it's time to stop
        tx.send(()).await.expect("failed to send signal");

        // Stop the event listener
        handle.abort();
    }
}
