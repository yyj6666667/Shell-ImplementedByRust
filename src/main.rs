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
      let exit_check: String = String::from("exit");
      if command.trim() == exit_check {
        break;
      }

      println!("{}: command not found", command.trim());
    }
}
