use std::{env::current_dir, fs::{self, remove_dir_all, File}, io::{BufRead, BufReader, BufWriter, Error, ErrorKind, Seek, SeekFrom, Write}, path::Path, process::{Command, ExitStatus}};

fn main() -> Result<(), Error>{
    let cwd = current_dir()?;

    let remote_origin = get_remote_origin(&cwd)?;

    update_indexhtml(&cwd.join("dist"), &remote_origin)?;

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


/// Updates index.html to use the correct file paths for gh-pages to work correctly. 
/// 
/// # Example
/// ```
/// let cwd = std::env::current_dir()?;
/// update_indexhtml(cwd.join("dist"), "https://github.com/FradulentUser/MyRepo.git")?;
/// ```
fn update_indexhtml(dist_path: &Path, remote_origin: &str) -> Result<(), Error>{
    let repo_name = remote_origin.rsplit_once(".git")
                                                .unwrap_or((remote_origin,"")).0
                                                .rsplit_once("/")
                                                .unwrap_or(("",remote_origin)).1;
    let dirs: Vec<String> = dist_path.read_dir()?
                                .map_while(Result::ok)
                                .filter_map(|dir| Some(dir.path().file_name()?.to_str()?.to_owned()))
                                .collect();
    
    let mut index_html = File::options().read(true).write(true).open(dist_path.join("index.html"))?;
    
    // modify index.html to use the correct file paths
    let read_lines: Vec<String> = BufReader::new(&index_html)
        .lines().into_iter().map_while(Result::ok)
        .map(|mut line| {
            for dir in dirs.iter() {
                if let Some(idx) = line.find(dir){line.insert_str(idx,&format!("{}/",repo_name))}
            }
            line
            }).collect();
    
    index_html.seek(SeekFrom::Start(0))?;
    index_html.write_all(read_lines.join("\n").as_bytes())?;

    Ok(())
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