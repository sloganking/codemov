use codevis::render::{BgColor, FgColor};
use glob::{glob, GlobError};
use image::{DynamicImage, GenericImageView, RgbaImage};
use kdam::tqdm;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
mod options;
use std::path::Path;

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

/// Resizes image to be a certain resolution. Adds invisible padding
/// around the image if it isn't large enough.
fn resize_image_at(path: &str, x: u32, y: u32) {
    let img = image::open(path).unwrap();
    let img = img.resize(x, y, image::imageops::FilterType::Nearest);
    let img = add_buffer_till_image_is(x, y, &img);

    img.save(path).unwrap();
}

/// Returns a list of all files in a directory and it's subdirectories
pub fn get_files_in_dir(path: &str, filetype: &str) -> Result<Vec<PathBuf>, GlobError> {
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
    let repo_link = &args.repo;
    let repo_branch = &args.branch;
    let repo_name = repo_link
        .split('/')
        .last()
        .expect("could not determine repo name.");

    // clean dir for repo cloning
    clean_dir(REPO_CLONING_DIR);
    // clean dir for frames
    clean_dir(IMG_OUTPUT_DIR);

    //> get list of commits

    // cd to where we will clone repo
    // can't run Command::new due to this
    // https://stackoverflow.com/questions/56895623/why-isnt-my-rust-code-cding-into-the-said-directory
    std::env::set_current_dir(REPO_CLONING_DIR).expect("Unable to change directory");

    // clone repo
    println!("Performing initial clone of repo");
    let _ = Command::new("git")
        .args(["clone", repo_link])
        .output()
        .unwrap();

    // cd into cloned dir
    std::env::set_current_dir("./".to_owned() + repo_name + "/")
        .expect("Unable to change directory");

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
    println!("Rendering commits...");
    for (i, commit) in tqdm!(commit_list_vec.iter().rev().enumerate()) {
        // git checkout <tag>
        let _ = Command::new("git")
            .args(["checkout", commit])
            .output()
            .unwrap();

        // configure how image should be rendered
        let opts = codevis::render::Options {
            column_width: 100,
            line_height: 2,
            readable: false,
            show_filenames: false,
            target_aspect_ratio: args.width as f64 / args.height as f64,
            threads: 0,
            highlight_truncated_lines: false,
            fg_color: FgColor::Style,
            bg_color: BgColor::Style,
            theme: "Solarized (dark)",
            force_full_columns: false,
            ignore_files_without_syntax: false,
            plain: false,
            display_to_be_processed_file: false,
            color_modulation: 0.3,
            tab_spaces: 4,
            line_nums: false,
        };

        let progress: Arc<prodash::Tree> = prodash::TreeOptions {
            message_buffer_capacity: false.then(|| 200).unwrap_or(20),
            ..Default::default()
        }
        .into();

        let should_interrupt = Arc::new(AtomicBool::new(false));
        let (paths, _ignored) = codevis::unicode_content(
            Path::new("./"),
            &Vec::new(),
            progress.add_child("search unicode files"),
            &should_interrupt,
        )
        .expect("failed to get list of files");

        let ss = SyntaxSet::load_defaults_newlines();
        let ts = ThemeSet::load_defaults();

        // create image
        // println!("rendering commit: {}", commit);
        let img = codevis::render(
            &paths,
            progress.add_child("render"),
            &should_interrupt,
            &ss,
            &ts,
            opts,
        )
        .expect("failed to render image");

        // save image
        let output_filename = &("../../frames/".to_owned() + &format!("{:09}", i) + ".png");
        img.save(output_filename).expect("failed to save file");
    }

    // move to frames directory
    std::env::set_current_dir("../../frames/").expect("Unable to change directory");

    // resize all images to desired video resolution
    println!("resizing images...");
    let paths = get_files_in_dir("./", "").unwrap();
    for path in tqdm!(paths.into_iter()) {
        resize_image_at(
            &path.into_os_string().into_string().unwrap(),
            args.width,
            args.height,
        );
    }

    // determine output_dir
    let output_dir = if Path::new(&args.output_dir).is_absolute() {
        args.output_dir
    } else {
        String::from("../") + &args.output_dir
    };

    // create video
    println!("generating video");
    let _ = Command::new("ffmpeg")
        .args([
            "-y",
            "-r",
            &args.fps.to_string(),
            "-f",
            "image2",
            "-pattern_type",
            "glob",
            "-i",
            "*.png",
            &output_dir,
        ])
        .output()
        .unwrap();

    if args.open {
        open::that(output_dir).expect("Could not open video");
    }
}
