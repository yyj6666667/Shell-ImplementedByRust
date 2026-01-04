#[allow(unused_imports)]
use std::io::{self, Write};

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
        "exit" => break,
        _ if command.starts_with("echo ") => {
            println!("{}", &command[5..]);
        }
        _ if command.starts_with("type") && command.len() > 4 => {
            let arg = &command[5..];
            if builtins.contains(&arg) { 
                println!("{} is a shell builtin", arg);
            }
        }
        _   => println!("{}: command not found", command),
      }
                        // let exit_check: String = String::from("exit");
                        //  if command == exit_check {
                        //    break;
                        //  }
    }
}
