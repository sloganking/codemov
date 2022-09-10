use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Erases all content of an existing directory, or creates an empty new one.
pub fn clean_dir(dir: &str) {
    // clear any existing output_dir
    if PathBuf::from(dir).is_dir() {
        fs::remove_dir_all(dir).unwrap();
    }
    fs::create_dir(dir).unwrap();
}

fn main() {
    println!("Hello, world!");

    const REPO_CLONING_DIR: &str = "./temp/";
    const IMG_OUTPUT_DIR: &str = "./frames/";
    let repo_link = "https://github.com/sloganking/codevis";
    let repo_branch = "master";
    let repo_name = repo_link.split('/').last().unwrap();

    // clean dir for repo cloning
    clean_dir(REPO_CLONING_DIR);
    // clean dir for frames
    clean_dir(IMG_OUTPUT_DIR);

    //> git list of commits

        // cd to where we will clone repo
        std::env::set_current_dir(REPO_CLONING_DIR).expect("Unable to change directory");

        // clone repo
        let _ = Command::new("git")
            .args(["clone", repo_link])
            .output()
            .unwrap();

        // cd into cloned dir
        // can't run Command::new due to this
        // https://stackoverflow.com/questions/56895623/why-isnt-my-rust-code-cding-into-the-said-directory
        std::env::set_current_dir("./".to_owned() + repo_name + "/")
            .expect("Unable to change directory");

        let branch_vec = Command::new("git")
            .args(["rev-list", repo_branch])
            .output()
            .unwrap()
            .stdout;

        let string = String::from_utf8(branch_vec).unwrap();

        let mut commit_list: Vec<&str> = string.split('\n').collect();

        // remove last empty line
        commit_list.pop();

        println!("last branch: {}", commit_list.iter().last().unwrap());

    //<

    for (i, commit) in commit_list.iter().rev().enumerate() {
        // git checkout <tag>
        println!("checking out: {}", commit);
        let out = Command::new("git")
            .args(["checkout", commit])
            .output()
            .unwrap();

        // create image
        let _ = Command::new("codevis")
            .args(["-i", "./", "-o", &("../../frames/".to_owned() + &format!("{:09}", i) + ".png")])
            .output()
            .unwrap();
    }

    // clean_dir("./images/");

    // let out = Command::new("cd").args(["codevis/"]).output().unwrap();

    // cd into cloned dir
    // can't run Command::new due to this
    // https://stackoverflow.com/questions/56895623/why-isnt-my-rust-code-cding-into-the-said-directory
    // std::env::set_current_dir("./codevis/").expect("Unable to change directory");

    // let out = Command::new("git")
    //     .args(["rev-list", "master"])
    //     .output()
    //     .unwrap();

    // std::env::set_current_dir("../").expect("Unable to change directory");

    // let out = Command::new("git")
    //     .args(["rev-list", "master"])
    //     .output()
    //     .unwrap();

    // println!("{:?}", out);

    // let test = out.stdout;

    // let str: String = &out.stdout.from_utf8();

    // let str = String::from_utf8(out.stdout).unwrap();

    // println!("{:?}",str);
}
