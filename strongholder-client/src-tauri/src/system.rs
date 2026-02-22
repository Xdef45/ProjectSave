use std::process::Stdio;
use tauri::{AppHandle, Manager, Runtime};
use tokio::io::AsyncWriteExt;
use tokio::process::Command as AsyncCommand; // Needed for piping to stdin asynchronously

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "linux")]
use std::fs;
#[cfg(target_os = "linux")]
use std::os::unix::fs::PermissionsExt;

// Define Windows process flags exactly once
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

// ==========================================
// OS HELPERS
// ==========================================

#[cfg(target_os = "windows")]
async fn run_wsl_async(args: &[&str]) -> Result<(), String> {
    let distro_name = get_ubuntu_distro_name().await;
    let output = AsyncCommand::new("wsl")
        .args(&["-d", &distro_name])
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .await
        .map_err(|e| format!("Failed to execute wsl process: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Err(format!(
            "WSL Error.\nSTDERR: {}\nSTDOUT: {}",
            if stderr.is_empty() {
                "No error output"
            } else {
                &stderr
            },
            if stdout.is_empty() {
                "No output"
            } else {
                &stdout
            }
        ))
    }
}

#[cfg(target_os = "windows")]
pub async fn get_ubuntu_distro_name() -> String {
    let output = AsyncCommand::new("wsl")
        .args(["--list", "--quiet"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .await;

    if let Ok(out) = output {
        let stdout_bytes = out.stdout;
        let clean_text = if stdout_bytes.contains(&0) && stdout_bytes.len() % 2 == 0 {
            let u16_chars: Vec<u16> = stdout_bytes
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .collect();
            String::from_utf16_lossy(&u16_chars)
        } else {
            String::from_utf8_lossy(&stdout_bytes).into_owned()
        };

        for line in clean_text.lines() {
            let trimmed = line.trim();
            if trimmed.to_lowercase().contains("ubuntu") {
                return trimmed.to_string();
            }
        }
    }
    "Ubuntu".to_string()
}

#[cfg(target_os = "windows")]
async fn translate_wsl_path(win_path: &str) -> String {
    let distro_name = get_ubuntu_distro_name().await;
    let output = AsyncCommand::new("wsl")
        .args(&["-d", &distro_name, "wslpath", "-a", win_path])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .await;

    if let Ok(out) = output {
        let bytes = out.stdout;
        let text = if bytes.contains(&0) && bytes.len() % 2 == 0 {
            let u16_chars: Vec<u16> = bytes
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .collect();
            String::from_utf16_lossy(&u16_chars)
        } else {
            String::from_utf8_lossy(&bytes).into_owned()
        };

        let trimmed = text.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    // Fallback
    let forward_slashes = win_path.replace("\\", "/");
    if let Some(colon_pos) = forward_slashes.find(':') {
        let drive = &forward_slashes[0..colon_pos].to_lowercase();
        let rest = &forward_slashes[colon_pos + 1..];
        format!("/mnt/{}{}", drive, rest)
    } else {
        forward_slashes
    }
}

// ==========================================
// SSH KEY HELPER
// ==========================================
async fn read_ssh_key_async(username: &str, file_name: &str) -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        let linux_path = format!("/home/{}/.ssh/{}", username, file_name);
        println!("[SSH Key] Attempting to read from WSL: {}", linux_path);

        let distro_name = get_ubuntu_distro_name().await;

        let output = AsyncCommand::new("wsl")
            .args(&["-d", &distro_name, "-u", username, "cat", &linux_path])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .await
            .map_err(|e| format!("Failed to run wsl command: {}", e))?;

        if output.status.success() {
            let content = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if content.starts_with("ssh-") {
                Ok(content)
            } else {
                Err(format!("File read but content invalid: {}", content))
            }
        } else {
            let err = String::from_utf8_lossy(&output.stderr);
            Err(format!("SSH Key not found. WSL Error: {}", err))
        }
    }

    #[cfg(target_os = "linux")]
    {
        let mut key_path = dirs::home_dir().ok_or("Could not find home directory")?;
        key_path.push(".ssh");
        key_path.push(file_name);

        if !key_path.exists() {
            return Err(format!("SSH key not found at: {:?}", key_path));
        }

        tokio::fs::read_to_string(key_path)
            .await
            .map_err(|e| e.to_string())
    }
}

// ==========================================
// TAURI COMMANDS
// ==========================================

#[tauri::command]
pub async fn save_master_key(
    username: String,
    client_id: String,
    key: Vec<u8>,
) -> Result<(), String> {
    let file_subpath = format!(".config/borg/keys/{}.gpg", client_id);
    let dir_subpath = ".config/borg/keys";

    #[cfg(target_os = "windows")]
    {
        let linux_file_path = format!("/home/{}/{}", username, file_subpath);
        let linux_dir_path = format!("/home/{}/{}", username, dir_subpath);

        println!("[Rust] Saving Master Key to WSL: {}", linux_file_path);

        let setup_cmd = format!("mkdir -p '{0}' && chmod 700 '{0}'", linux_dir_path);
        run_wsl_async(&["-u", &username, "--", "sh", "-c", &setup_cmd]).await?;

        let distro_name = get_ubuntu_distro_name().await;

        let mut child = AsyncCommand::new("wsl")
            .args(&[
                "-u",
                &username,
                "-d",
                &distro_name,
                "--",
                "tee",
                &linux_file_path,
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn WSL write process: {}", e))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(&key)
                .await
                .map_err(|e| format!("Failed to pipe data to WSL: {}", e))?;
        }

        let output = child
            .wait_with_output()
            .await
            .map_err(|e| format!("Failed to wait on WSL process: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("WSL Write Failed: {}", stderr));
        }

        let chmod_cmd = format!("chmod 600 '{}'", linux_file_path);
        run_wsl_async(&["-u", &username, "--", "sh", "-c", &chmod_cmd]).await?;
    }

    #[cfg(target_os = "linux")]
    {
        let mut target_path = dirs::home_dir().ok_or("Could not find home directory")?;
        target_path.push(dir_subpath);

        if !target_path.exists() {
            tokio::fs::create_dir_all(&target_path)
                .await
                .map_err(|e| e.to_string())?;

            let mut dir_perms = std::fs::metadata(&target_path)
                .map_err(|e| e.to_string())?
                .permissions();
            dir_perms.set_mode(0o700);
            std::fs::set_permissions(&target_path, dir_perms)
                .map_err(|e| format!("Failed to set dir permissions: {}", e))?;
        }

        target_path.push(format!("{}.gpg", client_id));
        println!("[Rust] Saving Master Key to Linux: {:?}", target_path);

        tokio::fs::write(&target_path, &key)
            .await
            .map_err(|e| format!("Failed to write master key file: {}", e))?;

        let mut file_perms = std::fs::metadata(&target_path)
            .map_err(|e| e.to_string())?
            .permissions();
        file_perms.set_mode(0o600);
        std::fs::set_permissions(&target_path, file_perms)
            .map_err(|e| format!("Failed to set file permissions: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
pub async fn get_tunnel_ssh_key(username: String, client_id: String) -> Result<String, String> {
    read_ssh_key_async(&username, &format!("borg_{}_tunnel_key.pub", client_id)).await
}

#[tauri::command]
pub async fn get_borg_ssh_key(username: String, client_id: String) -> Result<String, String> {
    read_ssh_key_async(&username, &format!("borg_{}_key.pub", client_id)).await
}

#[tauri::command]
pub async fn save_server_ssh_key(username: String, ssh_key: String) -> Result<(), String> {
    let file_name = "ssh_strongholder_server.pub";

    #[cfg(target_os = "windows")]
    {
        let linux_path = format!("/home/{}/.ssh/{}", username, file_name);
        let linux_dir = format!("/home/{}/.ssh", username);

        let setup_cmd = format!("mkdir -p '{0}' && chmod 700 '{0}'", linux_dir);
        run_wsl_async(&["-u", &username, "--", "sh", "-c", &setup_cmd]).await?;

        let distro_name = get_ubuntu_distro_name().await;

        let mut child = AsyncCommand::new("wsl")
            .args(&[
                "-u",
                &username,
                "-d",
                &distro_name,
                "--",
                "tee",
                &linux_path,
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn WSL write process: {}", e))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(ssh_key.as_bytes())
                .await
                .map_err(|e| format!("Failed to write key: {}", e))?;
        }

        let output = child
            .wait_with_output()
            .await
            .map_err(|e| format!("Failed to wait: {}", e))?;
        if !output.status.success() {
            return Err(format!(
                "WSL Write Failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let chmod_cmd = format!("chmod 600 '{}'", linux_path);
        run_wsl_async(&["-u", &username, "--", "sh", "-c", &chmod_cmd]).await?;
    }

    #[cfg(target_os = "linux")]
    {
        let mut ssh_dir = dirs::home_dir().ok_or("Could not find home directory")?;
        ssh_dir.push(".ssh");
        let file_path = ssh_dir.join(file_name);

        if !ssh_dir.exists() {
            tokio::fs::create_dir_all(&ssh_dir)
                .await
                .map_err(|e| format!("Failed to create .ssh dir: {}", e))?;

            let mut perms = std::fs::metadata(&ssh_dir)
                .map_err(|e| e.to_string())?
                .permissions();
            perms.set_mode(0o700);
            std::fs::set_permissions(&ssh_dir, perms)
                .map_err(|e| format!("Failed to secure .ssh dir: {}", e))?;
        }

        tokio::fs::write(&file_path, ssh_key)
            .await
            .map_err(|e| format!("Failed to write key file: {}", e))?;

        let mut perms = std::fs::metadata(&file_path)
            .map_err(|e| e.to_string())?
            .permissions();
        perms.set_mode(0o600);
        std::fs::set_permissions(&file_path, perms)
            .map_err(|e| format!("Failed to secure key file: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
pub fn restart_computer() {
    #[cfg(target_os = "windows")]
    {
        // Safe standard library command for fire-and-forget
        let _ = std::process::Command::new("shutdown")
            .args(["/r", "/t", "0"])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = std::process::Command::new("reboot").spawn();
    }
}

#[tauri::command]
pub async fn wsl_setup_user(username: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let check_cmd = format!("id -u '{}'", username);
        let exists = run_wsl_async(&["-u", "root", "sh", "-c", &check_cmd]).await;
        if exists.is_err() {
            let create_cmd = format!("useradd -m -s /bin/bash '{}'", username);
            run_wsl_async(&["-u", "root", "sh", "-c", &create_cmd])
                .await
                .map_err(|e| format!("Failed to create WSL user: {}", e))?;
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn wsl_provision_scripts<R: Runtime>(
    app: AppHandle<R>,
    username: String,
    client_id: String,
) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    let key1_path = format!("/home/{}/.ssh/borg_{}_key", username, client_id);
    #[cfg(target_os = "windows")]
    let key2_path = format!("/home/{}/.ssh/borg_{}_tunnel_key", username, client_id);

    let mut keys_exist = false;

    #[cfg(target_os = "windows")]
    {
        let distro_name = get_ubuntu_distro_name().await;
        let status = AsyncCommand::new("wsl")
            .args(&[
                "-d",
                &distro_name,
                "-u",
                &username,
                "test",
                "-f",
                &key1_path,
                "-a",
                "-f",
                &key2_path,
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .status()
            .await
            .map_err(|e| e.to_string())?;
        if status.success() {
            keys_exist = true;
        }
    }

    #[cfg(target_os = "linux")]
    {
        let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
        let native_key1 = home_dir.join(format!(".ssh/borg_{}_key", client_id));
        let native_key2 = home_dir.join(format!(".ssh/borg_{}_tunnel_key", client_id));

        if native_key1.exists() && native_key2.exists() {
            keys_exist = true;
        }
    }

    if keys_exist {
        println!("[Provision] Keys already found. Skipping script execution.");
        return Ok(());
    }

    let resource_dir = app
        .path()
        .resource_dir()
        .map_err(|_| "Could not find resource dir")?;
    let script_dir = resource_dir.join("resources").join("scripts");

    if !script_dir.exists() {
        return Err(format!("Scripts folder not found at: {:?}", script_dir));
    }

    #[cfg(target_os = "windows")]
    {
        let mut win_path_str = script_dir.to_string_lossy().to_string();
        if win_path_str.starts_with(r"\\?\") {
            win_path_str = win_path_str[4..].to_string();
        }

        let source_path = translate_wsl_path(&win_path_str).await;

        let setup_cmd = format!(
            "mkdir -p /usr/local/sbin/scripts && cp -r '{0}'/* /usr/local/sbin/scripts/ && chmod +x /usr/local/sbin/scripts/*.sh && chown -R '{1}':'{1}' /usr/local/sbin/scripts", 
            source_path, username
        );
        run_wsl_async(&["-u", "root", "sh", "-c", &setup_cmd]).await?;

        let run_cmd = format!(
            "/usr/local/sbin/scripts/install_all_client.sh '{}' '{}'",
            username, client_id
        );
        run_wsl_async(&["-u", "root", "sh", "-c", &run_cmd]).await?;

        let chown_cmd = format!("chown -R '{0}':'{0}' '/home/{0}/.config'", username);
        run_wsl_async(&["-u", "root", "sh", "-c", &chown_cmd]).await?;
    }

    #[cfg(target_os = "linux")]
    {
        let output = AsyncCommand::new("whoami")
            .output()
            .await
            .map_err(|e| e.to_string())?;
        let system_user = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let source_path = script_dir.to_string_lossy();

        let setup_cmd = format!(
            "mkdir -p /usr/local/sbin/scripts && cp -r '{0}'/* /usr/local/sbin/scripts/ && chmod +x /usr/local/sbin/scripts/*.sh && chown -R '{1}':'{1}' /usr/local/sbin/scripts", 
            source_path, system_user
        );

        let setup_status = AsyncCommand::new("pkexec")
            .args(&["sh", "-c", &setup_cmd])
            .stdin(Stdio::null())
            .status()
            .await
            .map_err(|e| format!("Sudo prompt failed: {}", e))?;

        if !setup_status.success() {
            return Err("Root setup failed or user cancelled the password prompt.".to_string());
        }

        let run_cmd = format!(
            "/usr/local/sbin/scripts/install_all_client.sh '{}' '{}'",
            system_user, client_id
        );
        let run_status = AsyncCommand::new("pkexec")
            .args(&["sh", "-c", &run_cmd])
            .stdin(Stdio::null())
            .status()
            .await
            .map_err(|e| e.to_string())?;

        if !run_status.success() {
            return Err("Script execution failed".to_string());
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn wsl_configure_borg_client(username: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let wsl_key_path = format!("/home/{}/.ssh/ssh_strongholder_server.pub", username);
        let run_cmd = format!(
            "/usr/local/sbin/scripts/install_borghelper_key.sh '{}'",
            wsl_key_path
        );
        run_wsl_async(&["-u", "root", "sh", "-c", &run_cmd]).await?;
    }

    #[cfg(target_os = "linux")]
    {
        let home_dir = dirs::home_dir().ok_or("Could not find home directory")?;
        let lock_file = home_dir.join(".config/strongholder/.borg_configured");

        if lock_file.exists() {
            return Ok(());
        }

        let output = AsyncCommand::new("whoami")
            .output()
            .await
            .map_err(|e| e.to_string())?;
        let system_user = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let linux_key_path = format!("/home/{}/.ssh/ssh_strongholder_server.pub", system_user);

        let run_cmd = format!(
            "/usr/local/sbin/scripts/install_borghelper_key.sh '{}'",
            linux_key_path
        );
        let run_status = AsyncCommand::new("pkexec")
            .args(&["sh", "-c", &run_cmd])
            .stdin(Stdio::null())
            .status()
            .await
            .map_err(|e| format!("Borg configuration (sudo) failed: {}", e))?;

        if !run_status.success() {
            return Err("Borg configuration script failed or was cancelled.".to_string());
        }

        let config_dir = home_dir.join(".config/strongholder");
        if !config_dir.exists() {
            let _ = tokio::fs::create_dir_all(&config_dir).await;
        }
        let _ = tokio::fs::write(lock_file, "configured=true").await;
    }
    Ok(())
}

#[tauri::command]
pub async fn update_backup_schedule(
    username: String,
    preset_id: String,
    cron_string: String,
    preset_path: String,
    client_id: String,
    enabled: bool,
) -> Result<(), String> {
    let marker = format!("# STRONGHOLDER-ID:{}", preset_id);

    #[cfg(target_os = "windows")]
    let linux_path = translate_wsl_path(&preset_path).await;

    #[cfg(target_os = "linux")]
    let linux_path = preset_path;

    #[cfg(target_os = "linux")]
    let log_path = {
        let mut p = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
        p.push("strongholder_cron.log");
        p.to_string_lossy().into_owned()
    };

    #[cfg(target_os = "windows")]
    let log_path = format!("/home/{}/strongholder_cron.log", username);

    let backup_cmd = format!(
        "/usr/local/sbin/scripts/client_backup.sh '{}' '{}' >> '{}' 2>&1",
        client_id, linux_path, log_path
    );

    let new_cron_line = format!("{} {} {}", cron_string, backup_cmd, marker);
    let escaped_line = new_cron_line.replace("'", "'\\''");

    let sync_script = format!(
        "{{ \
            echo 'SHELL=/bin/bash'; \
            echo 'PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin'; \
            (crontab -l 2>/dev/null | grep -vE 'SHELL=|PATH=|{marker}' || true); \
            {append_logic} \
         }} > /tmp/cron_tmp && crontab /tmp/cron_tmp && rm /tmp/cron_tmp",
        marker = marker,
        append_logic = if enabled {
            format!("echo '{}';", escaped_line)
        } else {
            "".to_string()
        }
    );

    #[cfg(target_os = "windows")]
    {
        let distro_name = get_ubuntu_distro_name().await;
        let _ = AsyncCommand::new("wsl")
            .args(&["-d", &distro_name, "-u", "root", "service", "cron", "start"])
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .await;

        let status = AsyncCommand::new("wsl")
            .args(&[
                "-d",
                &distro_name,
                "-u",
                &username,
                "sh",
                "-c",
                &sync_script,
            ])
            .creation_flags(CREATE_NO_WINDOW)
            .status()
            .await
            .map_err(|e| e.to_string())?;

        if !status.success() {
            return Err("Failed to sync WSL crontab".into());
        }
    }

    #[cfg(target_os = "linux")]
    {
        let status = AsyncCommand::new("sh")
            .arg("-c")
            .arg(&sync_script)
            .status()
            .await
            .map_err(|e| e.to_string())?;
        if !status.success() {
            return Err("Failed to update crontab".into());
        }
    }

    Ok(())
}
