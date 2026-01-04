#[allow(unused_imports)]
use std::env;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::io::{self, Write};
use std::process::Command;

fn main() {
    // TODO: Uncomment the code below to pass the first stage
    let builtins = ["echo", "exit", "type"]; // 注意这里的元素类型是&str
                                                        // 调用contains 传入&&str

    loop {
      print!("$ ");
      io::stdout().flush().unwrap();
 
      //command not found 
      let mut command = String::new();
      io::stdin().read_line(&mut command).unwrap();
      //.trim() 去\n, 转成&str
      let command = command.trim();
      match command {
        "exit" => {
            break;
        }
        //echo
        _ if command.starts_with("echo ") => {
            println!("{}", &command[5..]);
        }
        //type
        _ if command.starts_with("type") && command.len() > 4 => {
            let arg = &command[5..];
            if builtins.contains(&arg) { 
                println!("{} is a shell builtin", arg);
            } else if (find_exec_in_path(arg).is_some())  {
                // is_some(), is_none, 学到了
                // 可以用if let 语法写， 但是这里不整复杂了
                // unwarp() 解包 Option<>
                let path = find_exec_in_path(arg).unwrap();
                println!("{} is {}", arg, path);
            }
            
            else {
                println!("{}: not found", arg);
            }
        }
        //not built-in command
        //maybe an exec or wrong
        _ => {
            //check if is an exec
            // 唉， 这里开销有点大
            let parts: Vec<&str> = command.split_whitespace().collect();
            if parts.is_empty() { continue; }

            let args = &parts[1..];
            if let Some(found) = find_exec_in_path(&parts[0]) {
                Command::new(&found).args(args).exec();
            } else {
                println!("{}: command not found", command);
            }      
        }
      }
                        // let exit_check: String = String::from("exit");
                        //  if command == exit_check {
                        //    break;
                        //  }
    }
}

fn find_exec_in_path(potential: &str) -> Option<String> {
    //获取PATH， 失败直接返回None
    let path_exists = env::var("PATH").ok()?; // var 返回Result<String, VarError> , .ok() 可以被Result类调用， .ok()返回的是Option<String>, ? 使得none被立即返回

    for dir in path_exists.split(':') {
        let full_path = format!("{}/{}", dir, potential);
        let path = Path::new(&full_path);
        if path.exists() && path.is_file() {
            // similar to python: os.path.exists(path), 只不过这里先创建了一个Path对象
            if let Ok(metadata) = path.metadata() {
                if metadata.permissions().mode() & 0o111 != 0 {
                    //为了匹配fn签名 ->Option<String> 必须加一层封装， 这就是rust严格的地方
                    return Some(full_path);
                }
            }
        }
    }

    None
}
