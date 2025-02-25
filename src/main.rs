mod local;

use std::env;

use anyhow::Result;

const FOLDER: &str = ".mid";

fn main() {
    let args = env::args().collect::<Vec<String>>();

    let repo = local::Repo::load();

    if args.len() == 1 {
        println!("Status");
    }

    let command = &args[1];

    match command.as_str() {
        "diff" => {
            println!("Difference of two commits");
        }
        "init" => {
            println!("Initializing repository");
        }
        "stage" => {
            println!("Staging file");
            // <path>+
        }
        "commit" => {
            println!("Commiting changes");
        }
        "branch" => {
            println!("Branching");
            // -c --checkout <commit or branch name>
            // -r --reset <commit or branch name>
            // -n --new <name>
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
            panic!("Unkown command {}", other);
        }
    }
}
