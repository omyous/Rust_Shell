use std::io::{self, Write};
use std::process::{Command, Stdio, Child};
use std::fmt;
use std::cell::RefCell;
use std::rc::Rc;
use std::env;
use std::collections::HashMap;


// -> EX5
///
/// the struct for management a process
/// 
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

///
/// the struct for management the job set
/// 
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

    let filtered_env : HashMap<String, String> =
    env::vars().filter(|&(ref k, _)|
        k == "TERM" || k == "TZ" || k == "LANG" || k == "PATH"
    ).collect();

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

        let command = remove_extra_space(&user_input);

        if user_input == "exit" {
            break;
        }else if user_input == "jobs"{
            println!("{}", jobs);
            continue;
        }

        // -> Ex5
        if command.starts_with("start"){
            
            let child : std::process::Child;

            if command.contains("|"){
                child = exec_with_pipe_on_backend(command.clone(), &filtered_env);
            }else {
                child = exec_simple_on_backend(command.clone(), &filtered_env);
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
                exec_with_pipe(command, &filtered_env);
            }else {
                exec_simple(command, &filtered_env);
            }
        }

    };


    Ok(())
}

///
/// exec a command simple on back end
/// 
fn exec_simple_on_backend(command : String, env : &HashMap<String, String>) -> std::process::Child{
    let args : Vec<&str> = command.split(" ").collect();

    let child = 
        match Command::new("cmd")
                    .arg("/C")
                    .args(args)
                    .env_clear()
                    .envs(env)
                    .spawn(){
                        Ok(c) => c,
                        Err(why) => panic!("failed to execute process : {}", why.to_string())
                    };
    child

}

///
/// exec a commande with pipe on back end
/// 
fn exec_with_pipe_on_backend(command : String, env : &HashMap<String, String>)  -> std::process::Child{
    let commands : Vec<&str> = command.split("|").collect();
            
    let command_first = remove_extra_space(commands[0]);
    let args_first : Vec<&str> = command_first.split(" ").collect();

    let mut child_process = 
        match Command::new("cmd")
                .arg("/C")
                .args(args_first)
                .stdout(Stdio::piped())
                .env_clear()
                .envs(env)
                .spawn(){
                    Ok(c) => c,
                    Err(why) => panic!("failed to execute process : {}", why.to_string())
                };


    for index in 1 .. (commands.len() - 1) {

        let command_center = remove_extra_space(commands[index]);

        let args_center : Vec<&str> = command_center.split(" ").collect();

        child_process =  
            match Command::new("cmd")
                .arg("/C")
                .args(args_center)
                .stdin(Stdio::from(child_process.stdout.expect("somethting wrong with stdin")))
                .stdout(Stdio::piped())
                .env_clear()
                .envs(env)
                .spawn(){
                    Ok(c) => c,
                    Err(why) => panic!("failed to execute process : {}", why.to_string())
                };

    }

    let command_last = remove_extra_space(commands[commands.len() - 1]);

    let args_last : Vec<&str> = command_last.split(" ").collect();


    let child = 
        match Command::new("cmd")
            .arg("/C")
            .args(args_last)
            .stdin(Stdio::from(child_process.stdout.expect("somethting wrong with stdin")))
            .env_clear()
            .envs(env)
            .spawn(){
                Ok(c) => c,
                Err(why) => panic!("failed to execute process : {}", why.to_string())
            };

    child
}

///
/// exec a simple command
/// 
fn exec_simple(command : String, env : &HashMap<String, String>){ //Ex3
    let args : Vec<&str> = command.split(" ").collect();

    let status = 
        match Command::new("cmd")
            .arg("/C")
            .args(args)
            .env_clear()
            .envs(env)
            .status(){
                Ok(c) => c,
                Err(why) => panic!("failed to execute process : {}", why.to_string())
            };
            
    if !status.success(){
                println!("exec failed");
    }
}

///
/// exec a commande with pipe
/// 
fn exec_with_pipe(command : String, env : &HashMap<String, String>){ //Ex4
    let commands : Vec<&str> = command.split("|").collect();
            
            let command_first = remove_extra_space(commands[0]);
            let args_first : Vec<&str> = command_first.split(" ").collect();

            let mut child_process = 
                match Command::new("cmd")
                        .arg("/C")
                        .args(args_first)
                        .stdout(Stdio::piped())
                        .env_clear()
                        .envs(env)
                        .spawn(){
                            Ok(c) => c,
                            Err(why) => panic!("failed to execute process : {}", why.to_string())
                        };


            for index in 1 .. (commands.len() - 1) {

                let command_center = remove_extra_space(commands[index]);

                let args_center : Vec<&str> = command_center.split(" ").collect();

                child_process =  
                    match Command::new("cmd")
                        .arg("/C")
                        .args(args_center)
                        .stdin(Stdio::from(child_process.stdout.expect("somethting wrong with stdin")))
                        .stdout(Stdio::piped())
                        .env_clear()
                        .envs(env)
                        .spawn(){
                            Ok(c) => c,
                            Err(why) => panic!("failed to execute process : {}", why.to_string())
                        };

            }

            let command_last = remove_extra_space(commands[commands.len() - 1]);

            let args_last : Vec<&str> = command_last.split(" ").collect();


            let status = 
                match Command::new("cmd")
                    .arg("/C")
                    .args(args_last)
                    .stdin(Stdio::from(child_process.stdout.expect("somethting wrong with stdin")))
                    .env_clear()
                    .envs(env)
                    .status(){
                        Ok(c) => c,
                        Err(why) => panic!("failed to execute process : {}", why.to_string())
                    };

            if !status.success(){
                println!("exec failed");
            }
}

/// remove the extra space with the commande
/// 
/// # Examples
/// 
/// ```
/// let command = "  dir  | findstr src  ";
/// let ret = remove_more_space(command);
/// 
/// assert_eq!("dir | findstr src".to_string(), ret);
/// ```
fn remove_extra_space(s : &str) -> String{
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