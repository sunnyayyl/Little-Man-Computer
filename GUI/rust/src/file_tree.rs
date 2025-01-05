use crate::watcher::{WatcherState, WatcherStruct};
use godot::classes::{ITree, TextEdit, Tree, TreeItem};
use godot::meta::AsObjectArg;
use godot::prelude::*;
use notify::Watcher;
use std::fs;
use std::path::Path;

#[derive(GodotClass)]
#[class(base=Tree)]
pub(crate) struct FileTree {
    watcher: WatcherState,
    base: Base<Tree>,
}
#[godot_api]
impl ITree for FileTree {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,
            watcher: WatcherState::Stopped,
        }
    }
    fn process(&mut self, _delta: f64) {
        match &self.watcher {
            WatcherState::Running(watcher) => {
                if watcher.need_refresh() {
                    godot_print!("Refreshing");
                    self.recreate_tree();
                }
            }
            WatcherState::Stopped => {
                // for testing
                self.setup_watcher(vec!["/tmp/watch_test".into(), "/tmp/watch_test2".into()]);
                self.recreate_tree();
            }
        }
    }
}
#[godot_api]
impl FileTree {
    fn setup_watcher(&mut self, path: Vec<GString>) {
        match &mut self.watcher {
            WatcherState::Running(watcher) => {
                watcher.unwatch_all().expect("Failed to unwatch all paths");
            }
            WatcherState::Stopped => {}
        }
        self.watcher =
            WatcherState::Running(WatcherStruct::new(path).expect("Failed to create watcher"));
    }
    #[func]
    fn recreate_tree(&mut self) {
        self.base_mut().clear();
        self.base_mut().set_hide_root(true);
        let root = self.base_mut().create_item().unwrap();
        match self.watcher {
            WatcherState::Running(ref watcher) => {
                for path in watcher.paths.clone() {
                    let s = path.to_string();
                    let mut folder = self
                        .base_mut()
                        .create_item_ex()
                        .parent(&root)
                        .done()
                        .unwrap();
                    folder.set_text(0, &s);
                    let path = Path::new(&s);

                    if path.is_dir() {
                        self.read_folder(path.to_str().unwrap(), &folder);
                    } else {
                        todo!("Add file reading")
                    }
                }
            }
            WatcherState::Stopped => {}
        }
    }
    fn read_folder(&mut self, path: &str, parent: impl AsObjectArg<TreeItem> + Clone) {
        let paths = fs::read_dir(path).expect("Failed to read directory");
        for path in paths {
            let path = path.expect("Failed to read path");
            let path = path.path();
            let mut child = self
                .base_mut()
                .create_item_ex()
                .parent(parent.clone())
                .done()
                .unwrap();
            child.set_meta("path", &Variant::from(path.to_str().unwrap()));
            child.set_text(0, path.file_name().unwrap().to_str().unwrap());
            if path.is_dir() {
                child.set_meta("is_file", &Variant::from(false));
                self.read_folder(path.to_str().unwrap(), &child);
            } else if path.is_file() {
                child.set_meta("is_file", &Variant::from(true));
            }
        }
    }
    #[func]
    fn cell_selected(&mut self) {
        godot_print!("Cell selected");
        if let Some(selected) = self.base().get_selected() {
            let path = selected.get_meta("path").to_string();
            let is_file: bool = selected.get_meta("is_file").to();
            if is_file {
                godot_print!("Opening file: {}", path);
                let file = fs::read_to_string(path).expect("failed to read file");
                let mut code_edit: Gd<TextEdit> = self.base_mut().get_node_as("../CodeEdit");
                code_edit.set_text(&file);
            }
        }
    }
}
