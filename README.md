# gh-pages updater for rust projects compiled with trunk
This project is a super basic script for updating the gh-pages branch from rust projects that were compiled with `trunk build --release`. 

## Usage
First, ensure that your rust project was created with `cargo new`, and contains a `dist` folder containing the result of the `trunk build --release` command. 
Additionally, you should have already set up a github repo for your project. 

Then, assuming the compiled executable of this project is in your `PATH`, you can simply run `trunk-ghpages` (or whatever you named the executable) in your terminal 
from the project directory to push to the gh-pages branch of your project's repo. 

## Install
Under releases, you can install an executable compiled for 64-bit windows. For other platforms, you clone this repo and compile locally by simply doing `cargo build --release`. 
