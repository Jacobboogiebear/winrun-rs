use std::{ process::{ Command, Output }, env::{ current_exe } };
use uuid::{ Uuid };
use chrono::prelude::*;

pub struct User {
    userdomain: String,
    password: String,
    pdefined: bool,
    kcorrect: bool,
    incorrect: bool
}

pub struct UserArgs {
    username: Option<String>,
    password: Option<String>,
    domain: Option<String>
}

impl From<(String)> for UserArgs {
    fn from(arg: String) -> Self {
        let mut exp: UserArgs = UserArgs::default();
        exp.password = Some(arg);
        return exp;
    }
}

impl From<(String, String)> for UserArgs {
    fn from(args: (String, String)) -> Self {
        let mut exp: UserArgs = UserArgs::default();
        exp.password = Some(args.0);
        exp.username = Some(args.1);
        return exp;
    }
}

impl From<(String, String, String)> for UserArgs {
    fn from(args: (String, String, String)) -> Self {
        let mut exp: UserArgs = UserArgs::default();
        exp.password = Some(args.0);
        exp.username = Some(args.1);
        exp.domain = Some(args.2);
        return exp;
    }
}

impl Default for UserArgs {
    fn default() -> Self {
        UserArgs {
            username: None,
            password: None,
            domain: None
        }
    }
}

impl Default for User {
    fn default() -> Self {
        User {
            userdomain: String::from_utf8(Command::new("powershell.exe").args(["$(Get-WMIObject -class Win32_ComputerSystem | select username).username"]).output().unwrap().stdout).unwrap().replace("\r\n", ""),
            password: String::from(""),
            pdefined: false,
            kcorrect: false,
            incorrect: false
        }
    }
}

impl std::fmt::Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut censored: String = String::from("");
        for _i in 0..self.password.len() {
            censored += "*";
        }
        return f.debug_struct("User")
            .field("userdomain", &self.userdomain)
            .field("password", &censored)
            .field("pdefined", &self.pdefined)
            .field("kcorrect", &self.kcorrect)
            .field("incorrect", &self.incorrect)
            .finish();
    }
}

impl User {
    pub fn create<A>(xargs: A) -> Self where A: Into<UserArgs> {
        let args: UserArgs = xargs.into();
        let mut exp: User = User::default();
        if args.username.is_some() && args.domain.is_none() {
            let domain: String = String::from_utf8(Command::new("powershell.exe").args(["$(Get-WMIObject -class Win32_ComputerSystem | select name).name"]).output().unwrap().stdout).unwrap().replace("\r\n", "");
            exp.userdomain = format!("{}\\{}", domain, args.username.unwrap());
        } else if args.username.is_some() && args.domain.is_some() {
            exp.userdomain = format!("{}\\{}", args.domain.unwrap(), args.username.unwrap());
        } else if args.username.is_none() && args.domain.is_none() {
            exp.userdomain = String::from_utf8(Command::new("powershell.exe").args(["$(Get-WMIObject -class Win32_ComputerSystem | select username).username"]).output().unwrap().stdout).unwrap().replace("\r\n", "");
        } else if args.username.is_none() && args.domain.is_some() {
            panic!("Invalid args, username is required if domain is defined");
        }
        exp.password = args.password.unwrap();
        exp.pdefined = true;
        exp.validate();
        return exp;
    }

    pub fn get_userdomain(&self) -> String {
        return self.userdomain.clone();
    }

    pub fn get_password(&self) -> String {
        return self.password.clone();
    }

    pub fn is_password_correct(&self) -> bool {
        return !self.incorrect;
    }

    pub fn retry_pasword(&mut self, pass: String) -> bool {
        self.password = pass;
        return self.validate();
    }

    fn validate(&mut self) -> bool {
        let mut exp: bool = false;
        let runtime: DateTime<Local> = Local::now().checked_add_signed(chrono::Duration::seconds(240)).unwrap();
        let runtime_str: String = format!("{:02}:{:02}", runtime.hour(), runtime.minute());
        let task_id: String = Uuid::new_v4().to_string().replace("-", "");
        let exe_name: String = current_exe().unwrap().file_name().unwrap().to_str().unwrap().to_string();
        let task_name: String = format!("{}_{}", exe_name, task_id);
        let test_output: Output = Command::new("powershell.exe").args([format!("schtasks.exe /create /v1 /ru \"{}\" /rp \"{}\" /tn \"{}_testid\" /sc ONCE /tr \"powershell.exe\" /st \"{}\"", &self.userdomain, &self.password, task_name, runtime_str)]).output().unwrap();
        if (test_output.stderr == vec![] && test_output.stdout != vec![]) {
            Command::new("powershell.exe").args([format!("schtasks.exe /delete /tn \"{}_testid\" /f", task_name)]).output().unwrap();
            exp = true;
            self.pdefined = true;
            self.kcorrect = true;
            self.incorrect = false;
        } else {
            self.pdefined = true;
            self.kcorrect = true;
            self.incorrect = true;
        }
        return exp;
    }
}