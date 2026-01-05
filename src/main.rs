    #[allow(unused_imports)]
    use std::env;
    use std::os::unix::fs::PermissionsExt;
    use std::os::unix::process::CommandExt;
    use std::path::Path;
    use std::io;
    use std::io::Write;
    use std::process::Command;
    use std::fs::OpenOptions;

    #[derive(PartialEq, Clone, Copy, Debug)]
    enum RedirectKind {
        None,            //不重定向
        StdoutOverwrite, // > or 1>
        StdoutAppend,    // >> or 1>>
        StderrOverwrite, // 2>
        StderrAppend,    // 2>>
    }

    fn main() {
        // TODO: Uncomment the code below to pass the first stage
        let builtins = ["echo", "exit", "type", "pwd", "cd"]; // 注意这里的元素类型是&str
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
            "pwd" => {
                println!("{}", env::current_dir().unwrap().display());
            }
            //cd
            _ if command.starts_with("cd") => {
                let target = if command.len() > 3 {
                    &command[3..]
                } else {
                    "~"
                };
                // dealing with ~
                let path = if target == "~" {
                    env::var("HOME").unwrap()
                } else if target.starts_with("~/") {
                    let home = env::var("HOME").unwrap();
                    format!("{}{}", home, &target[1..])
                } else {
                    target.to_string()
                };

                // 啊， 好吧， 寻址., .. 是set_current_dir 本来就有的特性
                if env::set_current_dir(&path).is_err() {
                    println!("cd: {}: No such file or directory", &path);
                }
            }
            //echo
            _ if command.starts_with("echo ") => {
                //refactor with split_redirect
                let (left, mut target, kind) = split_redirect(command);
                if kind == RedirectKind::None {
                    //正常echo
                    println!("{}", left[1..].join(" "));
                } else {
                    // > or >>

                    let content_to_write = format!("{}\n", 
                    left[1..].join(" "));

                    match redirect_choice {
                        // >>
                        true => {
                            if let Ok(mut fd) = OpenOptions::new().create(true).write(true).append(true).open(target.as_ref().unwrap()) {
                                let _ = fd.write_all(content_to_write.as_bytes());
                            } else {
                                eprint!("echo: cannot open {} for append", target.unwrap());
                            }
                        }
                        // >
                        false => {
                            if let Ok(mut fd) = OpenOptions::new().create(true).write(true).truncate(true).open(target.as_mut().unwrap()) {
                                let _ = fd.write_all(content_to_write.as_bytes());
                            } else {
                                eprint!("echo: cannot open {} for overwrite", target.unwrap());
                            }
                        }
                    }
                }
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
            //extern command
            //maybe an exec or wrong
            _ => {
                //解构命令, 后面三个分支都会复用
                let (left, target, redirectKind) = split_redirect(command);

                if left.is_empty() { continue;}
                let cmd = left[0].as_str();
                let args = &left[1..];
                if let Some(path) = find_exec_in_path(cmd) {
                    if redirectKind == RedirectKind::None {
                        //外部命令 + 非重定向
                        Command::new(path).arg0(cmd).args(args).status().unwrap();
                    } else {
                        // > or >>
    
                        match redirectKind {
                            // >>
                            RedirectKind::StdoutAppend | RedirectKind::StderrAppend => {
                                if let Ok(output) = Command::new(&path).arg0(cmd).args(args).output() {
                                    let written_path = target.unwrap();
                                    if let Ok(mut fd) = OpenOptions::new()
                                        .create(true).write(true).append(true).open(&written_path) {
                                            let _ = fd.write_all(&output.stdout);
                                    } else {
                                        eprintln!("open {} failed", written_path);
                                    }
                                }                      
                            }
                            // >
                            RedirectKind::StderrOverwrite | RedirectKind::StdoutOverwrite => {
                                if let Ok(output) = Command::new(&path).arg0(cmd).args(args).output() {
                                    let written_path = target.unwrap();
                                    if let Ok(mut fd) = OpenOptions::new()
                                        .create(true).write(true).append(false).truncate(true).open(&written_path) {
                                            let _ = fd.write_all(&output.stdout);
                                    } else {
                                        eprintln!("open {} failed", written_path);
                                    }

                                    //debug: 可能第i个参数open err
                                    if !output.stderr.is_empty() {
                                        eprint!("{}", String::from_utf8_lossy(&output.stderr));
                                    }
                                }                      
                            }
                        }
                    }           
                } else {
                    println!("{}: command not found", command);
                }
            }
        }
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
/// 第一个bool用来区分append和overwri， 第二个bool用来区分是否是redirect
    fn split_redirect(input: &str) -> (Vec<String>, Option<String> , RedirectKind) {

        let normalize_target = |raw : &str| -> String {
            raw.trim().trim_matches('"').trim_matches('\'').to_string()
        };

        // 先尝试匹配">>"", pos 是模式匹配时(match, if let , while let), 当场创建的对象
        // 2>>
        match input.rfind("2>>") {
            Some(pos) => {
                let (left, right) = input.split_at(pos);
                let target = normalize_target(right[3..].trim());
                return (tokenize(left), Some(target), RedirectKind::StderrAppend);
            }
            None => {
                //continue
            }
        }
        // 1>>
        match input.rfind("1>>") {
            Some(pos) => {
                let (left, right) = input.split_at(pos);
                let target = normalize_target(right[3..].trim());
                return (tokenize(left), Some(target), RedirectKind::StdoutAppend);
            }
            None      => {
                //继续往下执行
            }       
        }
        // >>
        match input.rfind(">>") {
            Some(pos) => {
                let (left, right) = input.split_at(pos);
                let target = normalize_target(right[2..].trim());
                return (tokenize(left), Some(target), RedirectKind::StdoutAppend);
            }
            None      => {
                //继续往下执行
            }
        }

        // 再尝试匹配">"
        // 2>
        if let Some(pos) = input.rfind("2>") {
            let (left, right) = input.split_at(pos);
            let target = normalize_target(right[2..].trim());
            return (tokenize(left), Some(target), RedirectKind::StderrOverwrite);
        } 

        // 1>
        if let Some(pos) = input.rfind("1>") { 
            let (left, right) = input.split_at(pos);
            let target = normalize_target(right[2..].trim());
            return (tokenize(left), Some(target), RedirectKind::StdoutOverwrite);
        } 

        // >
        if let Some(pos) = input.rfind(">") { 
            let (left, right) = input.split_at(pos);
            let target = normalize_target(right[1..].trim());
            (tokenize(left), Some(target), RedirectKind::StdoutOverwrite)
        } else {
        // non-redirect
            return (tokenize(input), None, RedirectKind::None);
        }
    }

    fn tokenize(input: &str) -> Vec<String> {
        let mut args = Vec::new();
        let mut buf = String::new();
        let mut in_quotes = false;

        for char in input.chars() {
            match char {
                //reverse state
                '"' | '\'' => {
                    in_quotes = !in_quotes;
                }
                //碰到空格 而且 不在引用内， 触发添加args的条件
                c if c.is_whitespace() && in_quotes == false => {
                    args.push(buf.clone()); // 哈， 所有权
                    buf.clear();
                }
                //in normal case, add char to each arg, temperally stored in buf
                _ => {
                    buf.push(char);
                }
            }
        }
        //add last arg manually
        if !buf.is_empty() {
            args.push(buf.clone());
        }        

        args
    }
