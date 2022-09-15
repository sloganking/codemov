# codemov
 
**codemov** creates a video of how a git repository's code changes over time. Looking for an image instead of a video? Check out this repo's sister project [codevis](https://github.com/sloganking/codevis).
 
![](./assets/prodash.gif)

The result of running this repository on [prodash](https://github.com/Byron/prodash)

## Requirements
This repo currently makes use of calling some CLI programs. In order to run it, you must have
- Installed `git`
- Installed `ffmpeg`

For convenience, some of these may become cargo dependencies at a later date.

## Usage

To generate a video of this repo `https://github.com/sloganking/codemov` (though any will do)

Run `cargo run -- --repo https://github.com/sloganking/codemov --branch master`

For a list of further commands,

Run `cargo run -- --help`
