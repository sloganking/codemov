use glob::{glob, GlobError};
use image::{DynamicImage, GenericImageView, RgbaImage};
use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};
mod options;

/// Adds invisible padding around an image so it becomes the
/// requested resolution. The new image will be in the center
/// of the padding.
fn add_buffer_till_image_is(x: u32, y: u32, old_img: &DynamicImage) -> DynamicImage {
    if old_img.width() == x && old_img.height() == y {
        return old_img.clone();
    }

    let mut new_img = RgbaImage::new(x, y);

    // assert new dimentions are same or bigger.
    if old_img.width() > x {
        panic!();
    }
    if old_img.height() > y {
        panic!();
    }

    let new_x0 = (x - old_img.width()) / 2;
    let new_y0 = (y - old_img.height()) / 2;

    for tx in 0..old_img.width() {
        for ty in 0..old_img.height() {
            new_img.put_pixel(new_x0 + tx, new_y0 + ty, old_img.get_pixel(tx, ty));
        }
    }

    new_img.into()
}

fn resize_image_at(path: &str, x: u32, y: u32) {
    let img = image::open(path).unwrap();
    let img = img.resize(x, y, image::imageops::FilterType::Nearest);
    let img = add_buffer_till_image_is(x, y, &img);

    img.save(path).unwrap();
}

/// Returns a list of all files in a directory and it's subdirectories
pub fn get_files_in_dir(path: &str, filetype: &str) -> Result<Vec<PathBuf>, GlobError> {
    //> get list of all files and dirs in path, using glob
        let mut paths = Vec::new();

        let mut potential_slash = "";
        if PathBuf::from(path).is_dir() && !path.ends_with('/') {
            potential_slash = "/";
        }

        let search_params = String::from(path) + potential_slash + "**/*" + filetype;

        for entry in glob(&search_params).expect("Failed to read glob pattern") {
            match entry {
                Ok(path) => {
                    paths.push(path);
                }
                Err(e) => return Err(e),
            }
        }

    //<
    // filter out directories
    let paths = paths.into_iter().filter(|e| e.is_file()).collect();

    Ok(paths)
}

/// Erases all content of an existing directory, or creates an empty new one.
pub fn clean_dir(dir: &str) {
    // clear any existing output_dir
    if PathBuf::from(dir).is_dir() {
        fs::remove_dir_all(dir).unwrap();
    }
    fs::create_dir(dir).unwrap();
}

fn main() {
    let args: options::Args = clap::Parser::parse();

    const REPO_CLONING_DIR: &str = "./temp/";
    const IMG_OUTPUT_DIR: &str = "./frames/";
    // let repo_link = "https://github.com/sloganking/codevis";
    let repo_link = &args.repo;
    let repo_branch = "master";
    let repo_name = repo_link
        .split('/')
        .last()
        .expect("could not determine repo name.");

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

        let out = Command::new("git")
            .args(["rev-list", repo_branch])
            .output()
            .unwrap();
        // println!("out: {:?}", out);

        let commit_list_bytes = Command::new("git")
            .args(["rev-list", repo_branch])
            .output()
            .unwrap()
            .stdout;
        let commit_list_string = String::from_utf8(commit_list_bytes).unwrap();
        let mut commit_list_vec: Vec<&str> = commit_list_string.split('\n').collect();

        // remove last empty line
        commit_list_vec.pop();

    //<

    // render frame for each commit
    for (i, commit) in commit_list_vec.iter().rev().enumerate() {
        // git checkout <tag>
        println!("rendering commit: {}", commit);
        let out = Command::new("git")
            .args(["checkout", commit])
            .output()
            .unwrap();

        // create image
        let _ = Command::new("codevis")
            .args([
                "-i",
                "./",
                "-o",
                &("../../frames/".to_owned() + &format!("{:09}", i) + ".png"),
                "--force-full-columns",
                "false",
            ])
            .output()
            .unwrap();
    }

    // move to frames directory
    std::env::set_current_dir("../../frames/").expect("Unable to change directory");

    // resize all images to desired video resolution
    println!("resizing images...");
    let paths = get_files_in_dir("./", "").unwrap();
    for path in paths {
        resize_image_at(&path.into_os_string().into_string().unwrap(), 1920, 1080);
    }

    // println!("env::current_dir: {}", env::current_dir().unwrap().into_os_string().into_string().unwrap());

    //> generating video

        println!("generating video");

        // println!("out: {:?}", out);

        // create video
        // ffmpeg -y -r 30 -f image2 -pattern_type glob -i '*.png' output.mp4
        let out = Command::new("ffmpeg")
            .args([
                "-y",
                "-r",
                "30",
                "-f",
                "image2",
                "-pattern_type",
                "glob",
                "-i",
                "*.png",
                "../output.mp4",
            ])
            .output()
            .unwrap();

        // println!("out: {:?}", out);
    //<

    if args.open {
        open::that("../output.mp4").expect("Could not open video");
    }
}
