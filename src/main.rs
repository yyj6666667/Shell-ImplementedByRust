#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    // TODO: Uncomment the code below to pass the first stage
    loop {
      print!("$ ");
      io::stdout().flush().unwrap();
 
      //command not found 
      let mut command = String::new();
      io::stdin().read_line(&mut command).unwrap();
      //.trim() 去\n, 转成&str
      match command.trim() {
        "exit" => break,
        // _表示任意匹配
        _ if command.trim().starts_with("echo ") => println!("{}", &command.trim()[5..]),
        _ => println!("{}: command not found", command.trim()),
      }
    // let exit_check: String = String::from("exit");
    //  if command.trim() == exit_check {
    //    break;
    //  }
    }
}
