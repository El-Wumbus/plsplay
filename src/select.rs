use std::io::{stdin, stdout, Write};
use std::path::PathBuf;
use std::process::exit;

pub fn select_from_dir(path_list: Vec<PathBuf>) -> PathBuf
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
