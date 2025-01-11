use std::{env::current_dir, fs::{remove_dir_all, File}, io::{BufRead, BufReader, Error, ErrorKind}, path::Path, process::{Command, ExitStatus}};

fn main() -> Result<(), Error>{
    let cwd = current_dir()?;

    let remote_origin = get_remote_origin(&cwd)?;

    push_dir_to_branch(&remote_origin, &cwd.join("dist"), "gh-pages")?;

    Ok(())
}

/// Get the remote origin by checking ./.git/config in the current working directory
/// 
/// On success, returns Ok(String) where the string represents the URL of the remote origin\
/// On error, returns Err(std::io::Error)\
/// See std::io::Error for more info on the error return value.
/// # Example:
/// ```
/// let cwd = std::env::current_dir()?;
/// let remote_origin = get_remote_origin(&cwd)?;
/// ```
fn get_remote_origin(cwd: &Path) -> Result<String, Error>{
    // open config file in read mode
    let config_file = File::options()
        .read(true)
        .write(false)
        .open(cwd.join(".git/config"))?;
    
    // create a buffer to iterate through the lines of the config file
    let mut read_lines = BufReader::new(config_file)
        .lines().into_iter().map_while(Result::ok);

    // progress the iterator until the line containing [remote "origin"] is found 
    if read_lines.find(|line| line.contains("[remote \"origin\"]")).is_none(){
        return Err(Error::new(ErrorKind::NotFound, "Could not find remote origin in .git/config"))
    }

    // get the actual url, or return error if not found
    if let Some(line) = read_lines.find(|line| line.contains("url") || line.contains('[')){
        if line.contains("url"){
            if let Some((_, url)) = line.split_once('='){
                return Ok(url.trim().to_string())
            }
        }
    }
    Err(Error::new(ErrorKind::NotFound, "Could not find remote origin URL in .git/config"))
}


/// Commits and force pushes the contents of the specified directory to the given branch of the remote origin
/// 
/// # Example
/// ```
/// let cwd = current_dir()?;
/// push_dir_to_branch("https://github.com/FradulentUser/MyRepo.git", cwd.join("src"), "gh-pages")
/// ```
fn push_dir_to_branch(remote_origin: &str, dir: &Path, branch: &str) -> Result<(), Error>{
    let cmds_args: [&[&str]; 6] = [&["init"],
        &["remote","add","origin",remote_origin],
        &["add","."],
        &["commit","-am",&format!("Update {}",branch)],
        &["branch",branch],
        &["push","-uf","origin",branch]];

    for args in cmds_args{
        ensure_success(Command::new("git")
            .current_dir(dir)
            .args(args)
            .status())?;
    }

    remove_dir_all(dir.join(".git"))
}

/// Transforms a Result<ExitStatus, Error> so that Ok(ExitStatus) is only returned if the exit status is a success. 
/// Otherwise, returns an error.
/// 
/// # Example:
/// ```
/// let success_status = ensure_success(Command::new("ls").status()?);
/// ```
fn ensure_success(res: Result<ExitStatus, Error>) -> Result<ExitStatus, Error>{
    let res = res?;
    res.success().then(|| res).ok_or(Error::new(ErrorKind::Other, res.to_string()))
}