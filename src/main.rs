use codevis::render::{BgColor, FgColor, LineHighlight, LineHighlights};
use glob::{glob, GlobError};
use image::{DynamicImage, GenericImageView, RgbaImage};
use kdam::tqdm;
use std::collections::HashMap;
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

/// Represents the changes between two commits.
#[derive(Debug, Default)]
pub struct DiffInfo {
    /// Lines removed from old commit, indexed by file path -> set of 1-indexed line numbers
    pub removed_lines: HashMap<PathBuf, HashMap<usize, LineHighlight>>,
    /// Lines added in new commit, indexed by file path -> set of 1-indexed line numbers
    pub added_lines: HashMap<PathBuf, HashMap<usize, LineHighlight>>,
}

/// Parse git diff output to extract added and removed line numbers for each file.
/// Uses `git diff --unified=0` to get minimal context diff.
pub fn parse_git_diff(old_commit: &str, new_commit: &str) -> DiffInfo {
    let output = Command::new("git")
        .args(["diff", "--unified=0", old_commit, new_commit])
        .output()
        .expect("Failed to run git diff");

    let diff_str = String::from_utf8_lossy(&output.stdout);
    let mut info = DiffInfo::default();

    let mut current_file: Option<PathBuf> = None;

    for line in diff_str.lines() {
        // Parse file path from diff header
        // Format: +++ b/path/to/file
        if let Some(path) = line.strip_prefix("+++ b/") {
            current_file = Some(PathBuf::from(path));
            continue;
        }

        // Parse hunk header to get line numbers
        // Format: @@ -old_start,old_count +new_start,new_count @@
        // or: @@ -old_start +new_start @@ (for single line changes)
        if line.starts_with("@@") && line.contains("@@") {
            if let Some(ref file_path) = current_file {
                // Extract the hunk info between @@ markers
                let parts: Vec<&str> = line.split("@@").collect();
                if parts.len() >= 2 {
                    let hunk_info = parts[1].trim();
                    let chunks: Vec<&str> = hunk_info.split_whitespace().collect();

                    // Parse removed lines (-old_start,old_count or -old_start)
                    if let Some(removed) = chunks.iter().find(|s| s.starts_with('-')) {
                        if let Some((start, count)) = parse_hunk_range(removed.trim_start_matches('-')) {
                            let file_removed = info.removed_lines.entry(file_path.clone()).or_default();
                            for line_num in start..start + count {
                                file_removed.insert(line_num, LineHighlight::Removed);
                            }
                        }
                    }

                    // Parse added lines (+new_start,new_count or +new_start)
                    if let Some(added) = chunks.iter().find(|s| s.starts_with('+')) {
                        if let Some((start, count)) = parse_hunk_range(added.trim_start_matches('+')) {
                            let file_added = info.added_lines.entry(file_path.clone()).or_default();
                            for line_num in start..start + count {
                                file_added.insert(line_num, LineHighlight::Added);
                            }
                        }
                    }
                }
            }
        }
    }

    info
}

/// Parse a hunk range like "10,5" or "10" into (start_line, count).
fn parse_hunk_range(range: &str) -> Option<(usize, usize)> {
    if range.contains(',') {
        let parts: Vec<&str> = range.split(',').collect();
        if parts.len() == 2 {
            let start = parts[0].parse::<usize>().ok()?;
            let count = parts[1].parse::<usize>().ok()?;
            return Some((start, count));
        }
    } else {
        let start = range.parse::<usize>().ok()?;
        // Single line change
        return Some((start, 1));
    }
    None
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

    // render frame for each commit with diff highlighting
    println!("Rendering commits...");
    let commits: Vec<&str> = commit_list_vec.iter().rev().copied().collect();
    let mut frame_num: usize = 0;

    // Pre-load syntax and theme sets (they're expensive to create)
    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    // Helper to create base options
    let make_opts = |line_highlights: LineHighlights| codevis::render::Options {
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
        line_highlights,
    };

    // Helper to render and save a frame. Returns true if a frame was rendered, false if skipped.
    let render_frame = |commit: &str, line_highlights: LineHighlights, frame_num: usize, ss: &SyntaxSet, ts: &ThemeSet| -> bool {
        // git checkout the commit
        let _ = Command::new("git")
            .args(["checkout", commit])
            .output()
            .unwrap();

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

        // Skip if there are no files to render (empty commit or only binary files)
        if paths.children_content.is_empty() {
            return false;
        }

        let opts = make_opts(line_highlights);

        let img = codevis::render(
            &paths,
            progress.add_child("render"),
            &should_interrupt,
            ss,
            ts,
            opts,
        )
        .expect("failed to render image");

        let output_filename = format!("../../frames/{:09}.png", frame_num);
        img.save(&output_filename).expect("failed to save file");
        true
    };

    for (i, commit) in tqdm!(commits.iter().enumerate()) {
        if i == 0 {
            // First commit: just render normally (no previous commit to diff against)
            if render_frame(commit, HashMap::new(), frame_num, &ss, &ts) {
                frame_num += 1;
            }
        } else {
            let prev_commit = commits[i - 1];
            
            // Get diff between previous and current commit
            let diff_info = parse_git_diff(prev_commit, commit);

            // 1. Deletion frame: Show OLD commit with removed lines highlighted in red
            if !diff_info.removed_lines.is_empty() {
                if render_frame(prev_commit, diff_info.removed_lines, frame_num, &ss, &ts) {
                    frame_num += 1;
                }
            }

            // 2. Addition frame: Show NEW commit with added lines highlighted in green
            if !diff_info.added_lines.is_empty() {
                if render_frame(commit, diff_info.added_lines, frame_num, &ss, &ts) {
                    frame_num += 1;
                }
            }

            // 3. Normal frame: Show NEW commit without highlights
            if render_frame(commit, HashMap::new(), frame_num, &ss, &ts) {
                frame_num += 1;
            }
        }
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
