use notify::{Event, EventHandler, EventKind, RecommendedWatcher, Watcher};
use std::path::Path;
use std::sync::{Arc, RwLock};
use godot::builtin::GString;

#[derive(Debug)]
pub(crate) enum WatcherError {
    WatchError(notify::Error),
    UnwatchError(notify::Error),
    FileNotWatched,
}
type WatcherResult = Result<(), WatcherError>;
pub(crate) enum WatcherState {
    Stopped,
    Running(WatcherStruct),
}
pub(crate) struct WatcherStruct {
    watcher: RecommendedWatcher,
    pub paths: Vec<GString>,
    refresh: Arc<RwLock<bool>>,
}
impl WatcherStruct {
    pub fn new(paths: Vec<GString>) -> Result<Self, WatcherError> {
        let refresh = Arc::new(RwLock::new(false));
        let watcher = notify::recommended_watcher(EventChecker {
            refresh: refresh.clone(),
        })
        .unwrap();
        let mut new_struct = Self {
            watcher,
            refresh,
            paths: vec![],
        };
        new_struct.watch_all(paths)?;
        Ok(new_struct)
    }
    pub fn need_refresh(&self) -> bool {
        let mut refresh = self.refresh.write().unwrap();
        if *refresh {
            *refresh = false;
            true
        } else {
            false
        }
    }
    pub fn watch(&mut self, path: GString) -> WatcherResult {
        if let Err(error) = self.watcher.watch(Path::new(&path.to_string()), notify::RecursiveMode::Recursive) {
            Err(WatcherError::WatchError(error))
        } else {
            self.paths.push(path);
            Ok(())
        }
    }
    pub fn watch_all(&mut self, paths: Vec<GString>) -> WatcherResult {
        for path in &paths {
            if let Err(error) = self
                .watcher
                .watch(Path::new(&path.to_string()), notify::RecursiveMode::Recursive)
            {
                return Err(WatcherError::WatchError(error));
            }
        }
        self.paths.extend(paths);
        Ok(())
    }
    pub fn unwatch(&mut self, path: &Path) -> WatcherResult {
        let pos = self.paths.iter().position(|x| x.to_string() == path.to_str().unwrap());
        if let Some(pos) = pos {
            self.paths.remove(pos);
            if let Err(error) = self.watcher.unwatch(path) {
                Err(WatcherError::UnwatchError(error))
            } else {
                Ok(())
            }
        } else {
            Err(WatcherError::FileNotWatched)
        }
    }
    pub fn unwatch_all(&mut self) -> WatcherResult {
        for path in &self.paths {
            let r = self.watcher.unwatch(Path::new(&path.to_string()));
            if let Err(error) = r {
                return Err(WatcherError::UnwatchError(error));
            }
        }
        Ok(())
    }
}
struct EventChecker {
    refresh: Arc<RwLock<bool>>,
}
impl EventHandler for EventChecker {
    fn handle_event(&mut self, event: notify::Result<Event>) {
        if let Ok(event) = event {
            match event.kind {
                EventKind::Create(_) => {
                    let mut refresh = self.refresh.write().unwrap();
                    *refresh = true;
                }
                EventKind::Modify(_) => {
                    let mut refresh = self.refresh.write().unwrap();
                    *refresh = true;
                }
                EventKind::Remove(_) => {
                    let mut refresh = self.refresh.write().unwrap();
                    *refresh = true;
                }
                _ => {}
            }
        }
    }
}
