#[cfg(test)]
mod tests {
    use async_listener::file_system;
    use futures::pin_mut;
    use futures::stream::StreamExt;
    use std::time::Duration;

    #[tokio::test]
    async fn test_streaming_fs_watch() {
        // Create a temporary directory to use for the test
        let temp_dir = tempfile::tempdir().unwrap();

        // Create a file in the temporary directory
        let test_file1 = temp_dir.path().join("test11.txt");
        let test_file2 = temp_dir.path().join("test23.txt");
        let test_file3 = temp_dir.path().join("test33.txt");
        let test_file4 = temp_dir.path().join("test44.txt");

        let mut events = Vec::new();

        let handle = tokio::task::spawn(async move {
            // Create the stream and wait for it to produce some events
            let stream = file_system::streaming_fs_watch(temp_dir.path().to_path_buf()).await;
            pin_mut!(stream);

            // Wait for the stream to start
            tokio::time::sleep(Duration::from_millis(500)).await;

            std::fs::write(&test_file1, "test1").unwrap();
            std::fs::write(&test_file2, "test2").unwrap();
            std::fs::write(&test_file3, "test3").unwrap();
            std::fs::write(&test_file4, "test4").unwrap();

            while let Some(result) = stream.next().await {
                events.push(result.unwrap());
                if events.len() >= 8 {
                    break;
                }
            }

            assert_eq!(events.len(), 8usize);
        });

        let timeout = 5;

        let result = tokio::time::timeout(Duration::from_secs(timeout), handle).await;
        match result {
            Ok(_) => {
                // The task completed within the timeout
                return;
            }
            Err(_) => {
                // The task timed out
                panic!("Test timed out after {:?} seconds", timeout);
            }
        }
    }
}
