use std::env;

fn main() {
    let args = env::args().collect::<Vec<String>>();

    let command = &args[1];

    match command.as_str() {
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
            // -n --new <name>
            // -d --delete <name>
        }
        "merge" => {
            println!("Merging branches");
            // <commit or branch name 1> <commit or branch name 2>
        }
        "status" => {
            println!("Status");
        }
        other => {
            panic!("Unkown command {}", other);
        }
    }
}
