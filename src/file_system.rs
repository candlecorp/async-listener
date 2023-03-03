use crate::error::AsyncListenerError;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use tokio::sync::mpsc::{channel, Receiver};

// Define a new struct to represent file system events
#[derive(Debug)]
pub struct FileEvent {
    path: PathBuf,
    event: FileEventKind,
}

#[derive(Debug)]
enum FileEventKind {
    Created,
    Modified,
    Deleted,
    Accessed,
}

async fn async_watcher() -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let (tx, rx) = channel(1);

    let watcher = RecommendedWatcher::new(
        move |res| {
            let _ = tx.try_send(res); // send the event to the receiver without blocking
        },
        Config::default(),
    )?;

    Ok((watcher, rx))
}

pub async fn async_watch(path: PathBuf) -> Result<FileEvent, AsyncListenerError> {
    let (mut watcher, mut rx) = async_watcher().await?;

    watcher.watch(&path, RecursiveMode::Recursive)?;

    while let Some(res) = rx.recv().await {
        match res {
            Ok(event) => {
                let file_event = match event.kind {
                    notify::event::EventKind::Create(_) => FileEvent {
                        path: event.paths[0].clone(),
                        event: FileEventKind::Created,
                    },
                    notify::event::EventKind::Modify(_) => FileEvent {
                        path: event.paths[0].clone(),
                        event: FileEventKind::Modified,
                    },
                    notify::event::EventKind::Remove(_) => FileEvent {
                        path: event.paths[0].clone(),
                        event: FileEventKind::Deleted,
                    },
                    notify::event::EventKind::Access(_) => FileEvent {
                        path: event.paths[0].clone(),
                        event: FileEventKind::Accessed,
                    },
                    _ => continue, // Ignore other types of events
                };
                println!("file event: {:?}", file_event);
                return Ok(file_event);
            }
            Err(e) => return Err(AsyncListenerError::FileSystemError(format!("{}", e))),
        }
    }

    unreachable!(); // This line should never be reached
}
