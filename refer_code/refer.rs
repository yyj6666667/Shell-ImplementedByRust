
fn main() {
    let builtins = ["echo", "exit", "type", "pwd", "cd"]; 

    loop {
    print!("$ ");
    io::stdout().flush().unwrap();

    let mut command = String::new();
    io::stdin().read_line(&mut command).unwrap();
    let tokens = parse_args(command.trim());
    if tokens.is_empty() { continue; }

    let cmd = tokens[0].as_str();
    let args = &tokens[1..];

    match cmd {
        "exit" => break,
        "pwd"  => println!("{}", env::current_dir().unwrap().display()),
        "cd"   => { /* 用 args.get(0).unwrap_or(&"~".to_string()) 处理，~ 展开同原逻辑 */ }
        "echo" => {
            let (left, target, append, is_redir) = split_redirect_tokens(&tokens);
            let echo_args = &left[1..]; // 去掉 "echo"
            if !is_redir {
                println!("{}", echo_args.join(" "));
            } else {
                let content = format!("{}\n", echo_args.join(" "));
                if append {
                    if let Ok(mut fp) = OpenOptions::new().create(true).write(true).append(true).open(target.as_ref().unwrap()) {
                        let _ = fp.write_all(content.as_bytes());
                    } else {
                        eprintln!("echo: cannot open {} for append", target.unwrap());
                    }
                } else {
                    if let Ok(mut fp) = OpenOptions::new().create(true).write(true).truncate(true).open(target.as_ref().unwrap()) {
                        let _ = fp.write_all(content.as_bytes());
                    } else {
                        eprintln!("echo: cannot open {} for overwrite", target.unwrap());
                    }
                }
            }
        }
        "type" if args.len() >= 1 => {
            let arg = &args[0];
            if builtins.contains(&arg.as_str()) {
                println!("{arg} is a shell builtin");
            } else if let Some(path) = find_exec_in_path(arg) {
                println!("{arg} is {path}");
            } else {
                println!("{arg}: not found");
            }
        }
        _ => {
            let (left, target, append, is_redir) = split_redirect_tokens(&tokens);
            let cmd = left.get(0).map(|s| s.as_str());
            if cmd.is_none() { continue; }
            let cmd = cmd.unwrap();
            let args = left[1..].iter().map(|s| s.as_str());
            if let Some(path) = find_exec_in_path(cmd) {
                if !is_redir {
                    Command::new(path).arg0(cmd).args(args).status().unwrap();
                } else {
                    if let Ok(output) = Command::new(&path).arg0(cmd).args(args).output() {
                        let written_path = target.unwrap();
                        let mut opts = OpenOptions::new();
                        opts.create(true).write(true);
                        if append { opts.append(true); } else { opts.truncate(true); }
                        if let Ok(mut fp) = opts.open(&written_path) {
                            let _ = fp.write_all(&output.stdout);
                        } else {
                            eprintln!("open {} failed", written_path);
                        }
                    }
                }
            } else {
                println!("{cmd}: command not found");
            }
        }
    }
}
}

fn split_redirect_tokens(tokens: &[String]) -> (Vec<String>, Option<String>, bool, bool) {
    if tokens.len() >= 2 {
        let op = tokens[tokens.len() - 2].as_str();
        let file = tokens.last().unwrap().clone();
        let is_redir = matches!(op, ">" | "1>" | ">>" | "1>>");
        if is_redir {
            let append = op.ends_with(">>");
            return (tokens[..tokens.len() - 2].to_vec(), Some(file), append, true);
        }
    }
    (tokens.to_vec(), None, false, false)
}

fn parse_args(s: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut cur = String::new();
    let mut in_single = false;
    let mut in_double = false;
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\'' if !in_double => { in_single = !in_single; }
            '"'  if !in_single => { in_double = !in_double; }
            '\\' if !in_single => {
                if let Some(n) = chars.next() {
                    cur.push(n); // 仅在非单引号模式处理反斜杠
                }
            }
            _ if c.is_whitespace() && !in_single && !in_double => {
                if !cur.is_empty() {
                    args.push(std::mem::take(&mut cur));
                }
            }
            _ => cur.push(c),
        }
    }
    if !cur.is_empty() { args.push(cur); }
    args
}