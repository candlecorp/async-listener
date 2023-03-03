use crate::error::AsyncListenerError;
use futures::{
    channel::mpsc::{channel, Receiver},
    SinkExt, Stream, StreamExt,
};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};

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
    let (mut tx, rx) = channel(1);

    let watcher = RecommendedWatcher::new(
        move |res| {
            futures::executor::block_on(async {
                tx.send(res).await.unwrap();
            })
        },
        Config::default(),
    )?;

    Ok((watcher, rx))
}

pub async fn streaming_fs_watch(
    path: PathBuf,
) -> impl Stream<Item = Result<FileEvent, AsyncListenerError>> {
    let (mut watcher, mut rx) = async_watcher().await.unwrap();

    //don't use "try_stream" because that has its own Result<> that it wraps everything in and its
    //not compatible with the Result<> that we want to use
    async_stream::stream! {
        watcher.watch(&path, RecursiveMode::Recursive).unwrap();
        while let Some(res) = rx.next().await {
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
                    yield Ok(file_event);
                }
                Err(e) => yield Err(AsyncListenerError::FileSystemError(format!("{}", e))),
            }
        }
        unreachable!();
    }
}
