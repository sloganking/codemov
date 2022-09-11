# codemov
 
codemov creates a video of how a git repository's code changes over time. Looking for an image instead of a video? Check out this repo's sister project [codevis](https://github.com/sloganking/codevis).
 
![](./assets/prodash.gif)

The result of running this repository on [prodash](https://github.com/Byron/prodash)

## Requirements
This repo currently makes use of calling several CLI programs. In order to run it, you must have
- Installed `git`
- Installed `ffmpeg`
- Installed codevis `cargo install codevis@0.1.0`

For convenience, some of these may become cargo dependencies at a later date.

## Usage

To generate a video of this repo `https://github.com/sloganking/codemov`

Run `cargo run -- --repo https://github.com/sloganking/codemov`

For a list of further commands,

Run `cargo run -- --help`
