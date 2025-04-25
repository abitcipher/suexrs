use std::env;
use std::ffi::{CStr, CString};
use std::process::{Command, Stdio};
use libc::{c_int, getgrouplist, getgrnam, getpwnam, getpwuid, getuid, setgroups, setgid, setuid, gid_t, uid_t};

const MAX_GROUPS: usize = 100;
const SUEXRS_GROUP: &str = "suexrs";

// Check if the current user belongs to the suexrs group
fn user_in_suexrs_group() -> bool {
    let suexrs_group_cstr = CString::new(SUEXRS_GROUP).unwrap();
    let suexrs_group = unsafe { getgrnam(suexrs_group_cstr.as_ptr()) };
    if suexrs_group.is_null() {
        return false;
    }

    let suexrs_gid = unsafe { (*suexrs_group).gr_gid };

    let uid = unsafe { getuid() };
    if uid == 0 {
        return true;
    }

    let mut groups: [gid_t; MAX_GROUPS] = [0; MAX_GROUPS];
    let mut ngroups: c_int = MAX_GROUPS as c_int;

    let pwd = unsafe { getpwuid(uid) };
    if pwd.is_null() {
        return false;
    }

    let name_cstr = unsafe {
        CStr::from_ptr((*pwd).pw_name)
            .to_str()
            .unwrap_or_default()
    };
    let name_cstr = CString::new(name_cstr).unwrap();

    if unsafe { getgrouplist(name_cstr.as_ptr(), (*pwd).pw_gid, groups.as_mut_ptr(), &mut ngroups) } < 0 {
        eprintln!("Warning: Too many groups, may not validate all group memberships");
    }

    for group in groups.iter().take(ngroups as usize) {
        if *group == suexrs_gid {
            return true;
        }
    }

    false
}

// Parse a string in format [USER[:GROUP]] into user and group components
fn parse_user_group(arg: &str) -> Result<(String, Option<String>), String> {
    let parts: Vec<&str> = arg.split(':').collect();
    match parts.as_slice() {
        [user] => Ok((user.to_string(), None)),
        [user, group] => Ok((user.to_string(), Some(group.to_string()))),
        _ => Err(format!("Invalid user/group format: {}", arg)),
    }
}


// Set up supplementary groups for the target user
fn setup_groups(username: Option<&str>, target_gid: gid_t) -> Result<(), String> {
    let groups = match username {
        Some(name) => {
            let pwd_cstr = CString::new(name).unwrap();
            let pwd = unsafe { getpwnam(pwd_cstr.as_ptr()) };
            if pwd.is_null() {
                return Err(format!("Failed to find user '{}'", name));
            }
            let mut ngroups = MAX_GROUPS as c_int;
            let mut groups = vec![0; MAX_GROUPS];
            if unsafe { getgrouplist((*pwd).pw_name, (*pwd).pw_gid, groups.as_mut_ptr(), &mut ngroups) } < 0 {
                return Err(format!("Failed to get group list for user '{}'", name));
            }
            groups.truncate(ngroups as usize);
            groups
        }
        None => vec![target_gid],
    };

    if unsafe { setgroups((groups.len() as c_int).try_into().unwrap(), groups.as_ptr()) } < 0 {
        return Err("Failed to set supplemental groups".to_string());
    }

    Ok(())
}

fn main() {
    env_logger::init();
    log::info!("Application started");

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} [USER[:GROUP]] COMMAND [ARGUMENTS...]", args[0]);
        std::process::exit(1);
    }

    let user = match parse_user_group(&args[1]) {
        Ok((u, g)) => (u, g),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let cmd = &args[2..];
    if cmd.is_empty() {
        eprintln!("No command provided to execute");
        std::process::exit(1);
    }

    let is_root = unsafe { getuid() } == 0;
    let in_suexrs_group = user_in_suexrs_group();

    if !is_root && !in_suexrs_group {
        eprintln!("Permission denied: User not in '{}' group", SUEXRS_GROUP);
        std::process::exit(1);
    }

    let target_uid = match user.0.parse::<uid_t>() {
        Ok(uid) => uid,
        Err(_) => {
            let pwd_cstr = CString::new(user.0.as_str()).unwrap();
            let pwd = unsafe { getpwnam(pwd_cstr.as_ptr()) };
            if pwd.is_null() {
                eprintln!("Failed to find user '{}'", user.0);
                std::process::exit(1);
            }
            unsafe { (*pwd).pw_uid }
        }
    };

    let target_gid = match user.1 {
        Some(g) => {
            match g.parse::<gid_t>() {
                Ok(gid) => gid,
                Err(_) => {
                    let grp_cstr = CString::new(g.as_str()).unwrap();
                    let grp = unsafe { getgrnam(grp_cstr.as_ptr()) };
                    if grp.is_null() {
                        eprintln!("Failed to find group '{}'", g);
                        std::process::exit(1);
                    }
                    unsafe { (*grp).gr_gid }
                }
            }
        }
        None => {
            let pwd = unsafe { getpwuid(target_uid) };
            if pwd.is_null() {
                eprintln!("Failed to find user with UID {}", target_uid);
                std::process::exit(1);
            }
            unsafe { (*pwd).pw_gid }
        }
    };

    if let Err(e) = setup_groups(Some(&user.0), target_gid) {
        eprintln!("{}", e);
        std::process::exit(1);
    }

    if unsafe { setgid(target_gid) } < 0 {
        eprintln!("Failed to set GID to {}", target_gid);
        std::process::exit(1);
    }

    if unsafe { setuid(target_uid) } < 0 {
        eprintln!("Failed to set UID to {}", target_uid);
        std::process::exit(1);
    }

    let status = Command::new(&cmd[0])
        .args(&cmd[1..])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();

    match status {
        Ok(status) if status.success() => {}
        Ok(status) => {
            eprintln!("Command exited with status {}", status);
            std::process::exit(status.code().unwrap_or(1));
        }
        Err(e) => {
            eprintln!("Failed to execute command: {}", e);
            std::process::exit(1);
        }
    }
}