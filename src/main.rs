#![allow(dead_code)]
#![allow(unused_variables)]
// #![allow(unused_imports)]
mod local;
use local::*;

use std::env;

fn main() {
    let args = env::args().collect::<Vec<String>>();

    let repo;

    if args.len() == 1 {
        println!("Status");
        let repo = local::Repo::load().expect("Failed to load repo");
        println!(
            "JSON :\n{}",
            serde_json::to_string_pretty(&repo).expect("Failed to serialize repo")
        );
        return;
    }

    let command = &args[1];

    if command == "init" {
        repo = Repo::init().expect("Failed to initalize repo");
        repo.save().expect("Failed to save repo");
        return;
    }

    let mut repo = local::Repo::load().expect("Failed to load repo");

    match command.as_str() {
        "diff" => {
            println!("Difference of two commits");
        }
        "add" => {
            println!("Indexing");

            repo.index_paths(args[2..].into()).unwrap();
            // <path>+
        }
        "commit" => {
            // <msg>
            repo.commit_index(args[2].clone()).unwrap();
            println!("Commiting changes");
        }
        "branch" => {
            println!("Branching");
            // -c --checkout <commit or branch name>
            //    --reset <commit or branch name>
            //    --restore <commit or branch name>
            // -n --new <name>
            //    --rename <name>
            // -d --delete <name>
        }
        "remote" => {
            // change or update remote repository
            println!("Updating remote");
            // -u --update <url>
            // -r --remove
        }
        "clone" => {
            // clone from remote repository
            println!("Cloning remote");
            // <url>
        }
        "push" => {
            println!("Pushing changes to remote");
        }
        "pull" => {
            println!("Pulling changes from remote");
        }
        "merge" => {
            println!("Merging branches");
            // <commit or branch name 1> <commit or branch name 2>
        }
        "help" => {
            println!("Help");
        }
        other => {
            panic!("Unknown command '{other}'");
        }
    }

    repo.save().expect("Failed to save changes to repository");
}
