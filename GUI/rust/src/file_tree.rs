use crate::watcher::{WatcherState, WatcherStruct};
use godot::classes::{ITree, TextEdit, Tree, TreeItem};
use godot::meta::AsObjectArg;
use godot::prelude::*;
use std::{fs, process};
use std::io::{BufRead, BufReader, Cursor, Lines};
use std::path::Path;
use shared::{lexer, Mailbox};
use shared::lexer::{Lexer, LexerResult, LineStructure};
use shared::assembler::{Assembler, State};
use shared::error::AssemblerError;
use shared::runtime::{Runtime, RuntimeCommon, RuntimeState};

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
                // restart watcher
                self.setup_watcher(vec![]);
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
                child.set_collapsed(true);
                self.read_folder(path.to_str().unwrap(), &child);
            } else if path.is_file() {
                child.set_meta("is_file", &Variant::from(true));
            }
        }
    }
    #[func]
    fn cell_selected(&mut self) {
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
    #[func]
    fn watch(&mut self, path: GString) {
        match &mut self.watcher {
            WatcherState::Running(watcher) => {
                godot_print!("Watching: {}", path);
                watcher.watch(path).expect("Failed to watch");
            },
            WatcherState::Stopped => {
                godot_error!("Watcher not running, failed to watch");
            }
        }
        self.recreate_tree();
    }
    #[func]
    fn run_button(&mut self) {
        let mut mailbox=Mailbox::new();
        let mut code_edit: Gd<TextEdit> = self.base_mut().get_node_as("../CodeEdit");
        let code = code_edit.get_text();
        godot_print!("Running code: {}", code);
        let mut lexer=Lexer::new(BufReader::new(Cursor::new(code.to_string())).lines());
        let result = (&mut lexer)
            .collect::<Result<Vec<Option<LineStructure>>, AssemblerError>>();
        let label_lookup = lexer.get_label_lookup().clone();
        let mut lexer_result: LexerResult = [const { None }; 100];
        match result {
            Ok(result) => {
                for (i, line) in result.into_iter().enumerate() {
                    lexer_result[i] = line;
                }
            }
            Err(err) => {
                println!("{}", err);
                process::exit(1);
            }
        }
        let mut assembler = Assembler::new(Cursor::new(code.to_string()).lines(), label_lookup, lexer_result);
        loop {
            match assembler.parse_line() {
                State::Ok(opcode) => {
                    mailbox[assembler.current_line() - 1] =
                        opcode.to_numeric_representation();
                }
                State::Err(err) => {
                    println!("{}", err);
                    process::exit(1);
                }
                State::Done => break,
            }
        }
        let mut runtime = GUIRuntime::new(mailbox);
        runtime.start();
    }
}
pub struct GUIRuntime{
    pub common: RuntimeCommon
}
impl GUIRuntime{
    pub fn new(mailbox: Mailbox) -> Self {
        Self {
            common: RuntimeCommon {
                accumulator: 0,
                program_counter: 0,
                negative_flag: false,
                mailbox,
            }
        }
    }
}
impl Runtime for GUIRuntime {
    fn get_common(&self) -> &RuntimeCommon {
        &self.common
    }
    fn get_common_mut(&mut self) -> &mut RuntimeCommon {
        &mut self.common
    }
    fn inp(&mut self, _: Option<u16>) -> RuntimeState {
        godot_print!("Input requested");
        RuntimeState::Running
    }
    fn out(&mut self, _: Option<u16>) -> RuntimeState {
        godot_print!("{}", self.common.accumulator);
        RuntimeState::Running
    }
    fn sout(&mut self, _: Option<u16>) -> RuntimeState {
        let char =
            u8::try_from(self.common.accumulator).expect("Cannot be converted to ascii character") as char;
        godot_print!("{}", char);
        RuntimeState::Running
    }

    fn pop(&mut self, _addr: Option<u16>) -> RuntimeState {
        panic!("Pop not implemented");
    }

    fn push(&mut self, _addr: Option<u16>) -> RuntimeState {
        panic!("Push not implemented");
    }
}