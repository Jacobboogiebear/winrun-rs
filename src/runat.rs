#![allow(unused_variables, unused_imports)]
use chrono::prelude::*;
use std::io::Write;
use std::process::Command;
use std::time::{ SystemTime, UNIX_EPOCH, Duration };
use uuid::{ Uuid, uuid };
use std::env::{ current_exe, temp_dir };
use std::fs::File;

use super::user::User;

pub fn run(time: i64, args: String, user: User) {
    if !user.is_password_correct() {
        panic!("Password is incorrect");
    }
    let runtime: DateTime<Local> = Local::now().checked_add_signed(chrono::Duration::seconds(time * 60)).unwrap();
    let runtime_str: String = format!("{:02}:{:02}", runtime.hour(), runtime.minute());
    let deletetime: DateTime<Local> = runtime.checked_add_signed(chrono::Duration::seconds(60)).unwrap();
    let deletetime_str: String = format!("{:02}:{:02}", deletetime.hour(), deletetime.minute());
    let task_id: String = Uuid::new_v4().to_string().replace("-", "");
    let exe_name: String = current_exe().unwrap().file_name().unwrap().to_str().unwrap().to_string();
    let exe_path: String = format!("{:?}", current_exe().unwrap()).replace("\\\\", "\\").replace("\"", "");
    let task_name: String = format!("{}_{}", exe_name, task_id);
    let delete_task_name: String = format!("{}_{}_delete", exe_name, task_id);
    let tmp_file_path: String = format!("{}{}.ps1", temp_dir().to_string_lossy().to_string(), task_name.replace(".exe", ""));
    let mut tmp_file: File = File::create(&tmp_file_path).unwrap();
    let mut generated_script: String = format!("{} {}", exe_path, args);
    generated_script = [generated_script, format!("schtasks.exe /delete /tn \"{}\" /f", task_name)].join("\n");
    generated_script = [generated_script, format!("schtasks.exe /create /rl HIGHEST /ru \"{}\" /rp \"{}\" /sc ONCE /st \"{}\" /v1 /tn {} /tr \"powershell.exe -WindowStyle hidden -Command 'Remove-Item -Path {} ; schtasks.exe /delete /tn {} /f'\"", user.get_userdomain(), user.get_password(), deletetime_str, delete_task_name, &tmp_file_path, &delete_task_name)].join("\n");
    tmp_file.write_all(generated_script.as_bytes()).unwrap();
    Command::new("powershell.exe").args([format!("schtasks.exe /create /ru \"{}\" /rp \"{}\" /rl HIGHEST /sc ONCE /st \"{}\" /v1 /tn {} /tr \"powershell.exe -WindowStyle hidden -File {}\"", user.get_userdomain(), user.get_password(), runtime_str, task_name, tmp_file_path)]).output().unwrap();
}