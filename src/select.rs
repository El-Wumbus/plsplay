use std::fs::ReadDir;
use std::io::{stdin, stdout, Write};
use std::path::PathBuf;
use std::process::exit;

pub fn get_file(file: PathBuf) -> PathBuf
{
    let mut file = file.clone();
    if file.is_dir()
    {
        loop
        {
            file = file.join(select_from_dir(dir_to_filename_list(
                file.read_dir().expect(
                    format!("Error: cannot read from dir '{}'", file.to_string_lossy()).as_str(),
                ),
            )));

            if file.is_file()
            {
                break;
            }
        }
    }
    return file;
}

fn select_from_dir(path_list: Vec<PathBuf>) -> PathBuf
{
    println!("Select a file to play:");
    let mut i: u32 = 0;

    for path in path_list.clone()
    {
        println!("{} [{}]", path.to_string_lossy(), i);
        i += 1;
    }

    let choice: u32;

    loop
    {
        let mut input = String::new();
        print!("Enter your choice: ");
        stdout().flush().expect("Error: Cannot flush stdout");
        stdin()
            .read_line(&mut input)
            .expect("Error: Cannot read from stdin");
        if input.trim() == "exit" || input.trim() == "quit"
        {
            exit(0);
        }
        println!("");
        match input.trim().parse()
        {
            Ok(x) =>
            {
                if x >= path_list.clone().len() as u32
                {
                    eprintln!("Error: Invalid choice: {}", x);
                    continue;
                }
                choice = x;
                break;
            }
            Err(x) =>
            {
                eprintln!("Error: Invalid choice: {}", x);
                continue;
            }
        };
    }

    return path_list[choice as usize].clone();
}

fn dir_to_filename_list(dir: ReadDir) -> Vec<PathBuf>
{
    dir.map(|entry| {
        let entry_path = entry.unwrap().path();
        let file_name = entry_path.file_name().unwrap();
        let file_name_pathbuf = PathBuf::from(file_name);
        file_name_pathbuf
    })
    .collect::<Vec<PathBuf>>()
}
