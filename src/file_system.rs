use std::fs::FileType;
use std::path::{Path, PathBuf};
use tokio::fs::{self};
use tokio::stream::StreamExt;
use tokio::sync::mpsc::{channel, Sender};
use tracing::{error, info, instrument, span, Level};

pub struct FileSystemEvent {
    pub path: PathBuf,
    pub kind: FileSystemEventKind,
}

pub enum FileSystemEventKind {
    Created,
    Modified,
    Deleted,
    Renamed,
}

pub struct FileSystemWatcher {
    watched_files: Vec<PathBuf>,
    tx: Sender<FileSystemEvent>,
}

impl FileSystemWatcher {
    pub fn new(watched_files: Vec<PathBuf>) -> Self {
        let (tx, mut rx) = channel(100);

        for file in watched_files.iter() {
            let tx = tx.clone();
            tokio::spawn(async move {
                let mut watcher = tokio::fs::watcher(file).await.unwrap();
                loop {
                    match watcher.next().await {
                        Some(Ok(event)) => {
                            let path = event.path();
                            let kind = match event.kind() {
                                Ok(kind) => kind,
                                Err(e) => {
                                    error!(error = %e, "failed to get event kind");
                                    continue;
                                }
                            };

                            let kind = match kind {
                                notify::event::EventKind::Create(_) => {
                                    info!(file = %path.display(), "file created");
                                    FileSystemEventKind::Created
                                }
                                notify::event::EventKind::Modify(_) => {
                                    info!(file = %path.display(), "file modified");
                                    FileSystemEventKind::Modified
                                }
                                notify::event::EventKind::Remove(_) => {
                                    info!(file = %path.display(), "file deleted");
                                    FileSystemEventKind::Deleted
                                }
                                notify::event::EventKind::Rename(_, new_path) => {
                                    info!(old_file = %path.display(), new_file = %new_path.display(), "file renamed");
                                    FileSystemEventKind::Renamed
                                }
                                _ => continue,
                            };

                            let _ = tx
                                .send(FileSystemEvent {
                                    path: path.to_path_buf(),
                                    kind,
                                })
                                .await;
                        }
                        Some(Err(e)) => {
                            error!(error = %e, "error while watching file");
                        }
                        None => {
                            error!("watcher stream ended unexpectedly");
                            break;
                        }
                    }
                }
            });
        }

        Self { watched_files, tx }
    }

    pub async fn next_event(&mut self) -> Option<FileSystemEvent> {
        self.tx.recv().await.ok()
    }
}

#[tracing::instrument(skip(path), fields(path = %path.display()))]
async fn get_file_type(path: &Path) -> Option<FileType> {
    match tokio::fs::metadata(path).await {
        Ok(metadata) => Some(metadata.file_type()),
        Err(e) => {
            error!(error = %e, "failed to get file type");
            None
        }
    }
}

#[tracing::instrument(skip(watcher), fields(filters = %watcher.watched_files.join(",")))]
pub async fn listen_for_file_system_events(watcher: &mut FileSystemWatcher) {
    let mut regex_filters = Vec::new();
    let mut glob_filters = Vec::new();

    for filter in watcher.watched_files.iter() {
        if let Ok(regex) = Regex::new(filter) {
            regex_filters.push(regex);
        } else if let Ok(pattern) = Pattern::new(filter) {
            glob_filters.push(pattern);
        } else {
            error!(filter = %filter, "invalid filter pattern");
        }
    }

    while let Some(event) = watcher.next_event().await {
        let path = event.path.clone();

        // Check if the event path matches any of the watched files or directories
        if regex_filters
            .iter()
            .any(|regex| regex.is_match(path.to_str().unwrap_or("")))
            || glob_filters
                .iter()
                .any(|pattern| pattern.matches_path(&path))
        {
            match event.kind {
                FileSystemEventKind::Created => {
                    let file_type = get_file_type(&path).await;
                    info!(file = %path.display(), file_type = ?file_type, "file created");
                }
                FileSystemEventKind::Modified => {
                    info!(file = %path.display(), "file modified");
                }
                FileSystemEventKind::Deleted => {
                    info!(file = %path.display(), "file deleted");
                }
                FileSystemEventKind::Renamed => {
                    let old_path = path;
                    let new_path = match fs::read_link(&path).await {
                        Ok(new_path) => new_path,
                        Err(e) => {
                            error!(error = %e, "failed to get new path for renamed file");
                            continue;
                        }
                    };
                    info!(old_file = %old_path.display(), new_file = %new_path.display(), "file renamed");
                }
            }
        }
    }
}
