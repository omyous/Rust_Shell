use std::io::{self, Write};
use std::process::{Command, Stdio, Child};
use std::fmt;
use std::cell::RefCell;
use std::rc::Rc;

// -> EX5
#[derive(Clone, Debug)]
pub struct Job{
    pub job_id : u32,
    pub job_command : String,
    pub process : Rc<RefCell<Child>>
}

impl fmt::Display for Job {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {

        let status_decripte : String;
        let mut process_temp = self.process.borrow_mut();
        match process_temp.try_wait(){
            Ok(Some(status)) => status_decripte = "exited with code : ".to_string() + &status.code().expect("fail to exit").to_string(),
            Ok(None) => {
                status_decripte = "running".to_string();
            }
            Err(_) => status_decripte = "exited with error".to_string(),
        }


        write!(f, "{}  \"{}\"      {}", self.job_id, self.job_command, status_decripte)
    }
}

pub struct Jobs{
    pub table : Vec<Job>,
    pub count : i32
}

impl fmt::Display for Jobs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PID     Command              Status\n")?;
        for job in &self.table{
            write!(f, "{}\n", job)?;
        }

        Ok(())
    }
}

impl Jobs {
    fn new() -> Jobs{
        Jobs{ table : Vec::new(), count : 0}
    }

    fn push(&mut self, job : Job){
        self.table.push(job);
    }
}

// <- Ex5

fn main() -> std::io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();

    let mut jobs = Jobs::new();

    loop {
        let mut handle = stdout.lock();
        handle.write_all(b"> ")?;
        handle.flush()?;

        let mut user_input = String::with_capacity(256);
        // On prend une référence mutable
        stdin.read_line(&mut user_input)?;
        // `?` sert à « propager l'erreur » dans la fonction appellante
        // c'est mieux que de crash avec un unwrap ou expect ;)

        user_input = user_input.trim_end().trim_end().to_string();

        if user_input == "exit" {
            break;
        }else if user_input == "jobs"{
            println!("{}", jobs);
            continue;
        }

        let command = split_command_with_espace(&user_input);
        // -> Ex5
        if command.starts_with("start"){
            
            let child : std::process::Child;

            if command.contains("|"){
                child = exec_with_pipe_on_backend(command.clone());
            }else {
                child = exec_simple_on_backend(command.clone());
            }

            let child_id = child.id();

            let child_rc = Rc::new(RefCell::new(child));

            let job = Job{
                job_id : child_id, 
                job_command : command.clone(), 
                process : Rc::clone(&child_rc)};

            jobs.push(job);
            
            jobs.count = jobs.count + 1;

            println!("[{}] {} {:?}", jobs.count, child_id, command);
        // <- Ex5
        }else {
            if command.contains("|"){
                exec_with_pipe(command);
            }else {
                exec_simple(command);
            }
        }

    };


    Ok(())
}

fn exec_simple_on_backend(command : String) -> std::process::Child{
    let args : Vec<&str> = command.split(" ").collect();

    let child = Command::new("cmd")
                            .arg("/C")
                            .args(args)
                            .spawn()
                            .expect("failed to execute process");
            
    child

}

fn exec_with_pipe_on_backend(command : String)  -> std::process::Child{
    let commands : Vec<&str> = command.split("|").collect();
            
    let command_first = split_command_with_espace(commands[0]);
    let args_first : Vec<&str> = command_first.split(" ").collect();

    let mut parent_command = Command::new("cmd")
                                        .arg("/C")
                                        .args(args_first)
                                        .stdout(Stdio::piped())
                                        .spawn()
                                        .expect("failed to execute process");


    for index in 1 .. (commands.len() - 1) {

        let command_center = split_command_with_espace(commands[index]);

        let args_center : Vec<&str> = command_center.split(" ").collect();

        parent_command =  Command::new("cmd")
                                    .arg("/C")
                                    .args(args_center)
                                    .stdin(parent_command.stdout.unwrap())
                                    .stdout(Stdio::piped())
                                    .spawn()
                                    .expect("failed to execute process");

    }

    let command_last = split_command_with_espace(commands[commands.len() - 1]);

    let args_last : Vec<&str> = command_last.split(" ").collect();


    let child = Command::new("cmd")
                            .arg("/C")
                            .args(args_last)
                            .stdin(parent_command.stdout.unwrap())
                            .spawn()
                            .expect("failed to execute process");

    child
}

fn exec_simple(command : String){ //Ex3
    let args : Vec<&str> = command.split(" ").collect();

    let status = Command::new("cmd")
                            .arg("/C")
                            .args(args)
                            .status()
                            .expect("failed to execute process");
            
    if !status.success(){
                println!("exec failed");
    }
}

fn exec_with_pipe(command : String){ //Ex4
    let commands : Vec<&str> = command.split("|").collect();
            
            let command_first = split_command_with_espace(commands[0]);
            let args_first : Vec<&str> = command_first.split(" ").collect();

            let mut parent_command = Command::new("cmd")
                                        .arg("/C")
                                        .args(args_first)
                                        .stdout(Stdio::piped())
                                        .spawn()
                                        .expect("failed to execute process");


            for index in 1 .. (commands.len() - 1) {

                let command_center = split_command_with_espace(commands[index]);

                let args_center : Vec<&str> = command_center.split(" ").collect();

                parent_command =  Command::new("cmd")
                                    .arg("/C")
                                    .args(args_center)
                                    .stdin(parent_command.stdout.unwrap())
                                    .stdout(Stdio::piped())
                                    .spawn()
                                    .expect("failed to execute process");

            }

            let command_last = split_command_with_espace(commands[commands.len() - 1]);

            let args_last : Vec<&str> = command_last.split(" ").collect();


            let status = Command::new("cmd")
                            .arg("/C")
                            .args(args_last)
                            .stdin(parent_command.stdout.unwrap())
                            .status()
                            .expect("failed to execute process");

            if !status.success(){
                println!("exec failed");
            }
}


fn split_command_with_espace(s : &str) -> String{
    let mut index = 0;
    let mut ret : String = "".to_string();
    for i in s.chars(){
        if i != ' '{
            ret = s[index..].to_string();
            break;
        }
        index += 1;
    }

    let str_len = ret.len();

    if str_len < 2 {
        return "".to_string();
    }

    let mut index = 0;

    while index < (str_len - 2) {
        if ret.get(index .. (index + 2)) == Some("  ") {
            ret.remove(index + 1);
        }else {
            index += 1;
        }
    }

    let ret = ret.trim_end();

    ret.to_string()
}