use std::fs::read_dir;
use std::path::Path;
use std::process::Command;

fn main() {
    let mut blueprints =vec![];
    let files= read_dir("resources/ui").unwrap();
    for file in files{
        let file = file.unwrap();
        if file.file_type().unwrap().is_file() && file.path().extension().unwrap() == "blp"{
            blueprints.push(file.path().to_str().unwrap().to_string());
        }
    }
    let mut args =vec!["batch-compile".to_string(), "resources/generated".to_string(), "resources/ui".to_string()];
    args.extend(blueprints);
    println!("{:?}", args);
    Command::new("blueprint-compiler").args(args).status().unwrap();
}