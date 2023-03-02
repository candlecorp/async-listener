#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::{tempdir, TempDir};

    #[tokio::test]
    async fn test_listen_for_file_system_events() {
        // Create a temporary directory and file to watch
        let temp_dir = tempdir().expect("failed to create temp dir");
        let temp_file = temp_dir.path().join("test.txt");
        std::fs::write(&temp_file, "hello world").expect("failed to write to temp file");

        // Spawn a task to listen for file system events
        let handle = tokio::spawn(async move {
            listen_for_file_system_events(&temp_dir.path()).await;
        });

        // Wait for the event listener to start up
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Modify the file to trigger a file system event
        std::fs::write(&temp_file, "goodbye world").expect("failed to write to temp file");

        // Wait for the event to be processed
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Stop the event listener
        handle.abort();
    }
}
