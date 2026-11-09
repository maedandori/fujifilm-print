#[cfg(windows)]
pub mod app {
    use std::{fs, io::Read, path::{Path,PathBuf},process::{Command,Stdio},thread,time::Duration};
    use serde_json::Value;
    use sha2::{Digest,Sha256};

    const REPO: &str="https://api.github.com/repos/maedandori/fujifilm-print/releases/latest";
    const FILES: [&str;2] = ["Print-Agent.exe", "Print-Settings.exe"];
    fn log(s:&str){let path=std::env::current_exe().unwrap().parent().unwrap().join("rust-update.log");
        use std::io::Write;if let Ok(mut f)=fs::OpenOptions::new().create(true).append(true).open(path){let _=writeln!(f,"{s}");}}
    fn parse_version(s:&str)->Option<Vec<u64>>{
        let v=s.strip_prefix("rust-v").unwrap_or(s);
        v.split('.').map(|s|s.parse::<u64>().ok()).collect::<Option<Vec<_>>>()
    }
    fn executable_dir()->Result<PathBuf,String>{Ok(std::env::current_exe().map_err(|e|e.to_string())?.parent().ok_or("Thiếu thư mục EXE")?.to_path_buf())}
    fn fetch(url:&str,max:usize)->Result<Vec<u8>,String>{
        if !url.starts_with("https://github.com/maedandori/fujifilm-print/releases/download/")&&!url.starts_with("https://api.github.com/repos/maedandori/fujifilm-print/releases/"){
            return Err("URL cập nhật không thuộc repo Rust".into())
        }
        let request=ureq::AgentBuilder::new().timeout(Duration::from_secs(120)).build()
            .get(url).set("User-Agent","Fujifilm-Rust-Updater").set("Accept","application/octet-stream");
        let res=request.call().map_err(|e|e.to_string())?;
        let mut data=Vec::new();res.into_reader().take((max+1) as u64).read_to_end(&mut data).map_err(|e|e.to_string())?;
        if data.len()>max{return Err("Gói cập nhật quá lớn".into())}
        Ok(data)
    }
    fn process_running(image:&str)->bool{
        let filter=format!("IMAGENAME eq {image}");
        let result=Command::new("tasklist.exe").args(["/FI",&filter,"/FO","CSV","/NH"]).output();
        result.ok().is_some_and(|o|String::from_utf8_lossy(&o.stdout).to_ascii_lowercase().contains(&image.to_ascii_lowercase()))
    }
    fn core_running(dir:&Path)->bool{
        let path=dir.join("rust-core-active.pid");
        let pid=fs::read_to_string(path).ok().and_then(|v|v.trim().parse::<u32>().ok());
        pid.is_some_and(parent_running)
    }
    fn parent_running(pid:u32)->bool{
        let filter=format!("PID eq {pid}");
        Command::new("tasklist.exe").args(["/FI",&filter,"/FO","CSV","/NH"]).output()
          .ok().is_some_and(|o|String::from_utf8_lossy(&o.stdout).contains(&format!(",\"{pid}\",")))
    }
    pub fn check(tray_pid:u32)->Result<bool,String>{
        // Chỉ yêu cầu dừng Core nếu đã xác minh có bản cập nhật mới.
        let dir=executable_dir()?;
        let raw=ureq::AgentBuilder::new().timeout(Duration::from_secs(20)).build()
            .get(REPO).set("User-Agent","Fujifilm-Rust-Updater")
            .call().map_err(|e|e.to_string())?
            .into_string().map_err(|e|e.to_string())?;
        let meta:Value=serde_json::from_str(&raw).map_err(|e|e.to_string())?;
        let tag=meta["tag_name"].as_str().ok_or("Thiếu version GitHub Release")?;
        let latest=parse_version(tag).ok_or("Phiên bản release Rust không hợp lệ")?;
        let current=parse_version(option_env!("FUJIFILM_BUILD_VERSION").unwrap_or("0.0.0")).ok_or("Phiên bản hiện tại sai")?;
        if latest<=current {log("Đã ở phiên bản mới nhất");return Ok(false)}
        let assets=meta["assets"].as_array().ok_or("GitHub Release chưa có assets")?;
        let get_asset=|name:&str|->Result<String,String>{
            let asset=assets.iter().find(|a|a["name"]==name).ok_or(format!("Release thiếu {name}"))?;
            let url=asset["browser_download_url"].as_str().ok_or(format!("URL {name} không hợp lệ"))?;
            if !url.starts_with(&format!("https://github.com/maedandori/fujifilm-print/releases/download/{tag}/")) {
                return Err("Link asset sai repo hoặc sai tag".into())
            }
            Ok(url.to_string())
        };
        let sum_url=get_asset("fujifilm-rust-update.sha256")?;
        let zip_url=get_asset("fujifilm-rust-update.zip")?;
        let hash_text=String::from_utf8(fetch(&sum_url,1024)?).map_err(|e|e.to_string())?;
        let expected=hash_text.split_whitespace().next().ok_or("Thiếu SHA256")?;
        if expected.len()!=64||!expected.bytes().all(|b|b.is_ascii_hexdigit()){return Err("SHA256 không hợp lệ".into())}
        let bytes=fetch(&zip_url,120*1024*1024)?;
        if format!("{:x}",Sha256::digest(&bytes))!=expected.to_ascii_lowercase(){return Err("SHA256 gói cập nhật không khớp".into())}
        // Sau khi xác minh SHA-256, yêu cầu Core dừng hợp lệ giữa hai job.
        if core_running(&dir) {
            let stop=dir.join("rust-stop-request.flag");
            fs::write(&stop,"stop-for-update").map_err(|e|e.to_string())?;
            let mut stopped=false;
            for _ in 0..300 {
                if !core_running(&dir){stopped=true;break}
                thread::sleep(Duration::from_secs(1));
            }
            if !stopped {
                log("Core chưa dừng (có thể đang in). Hoãn cập nhật để tránh gián đoạn.");
                return Ok(false)
            }
        }
        // Không ghi file chương trình trước khi xác minh toàn bộ.
        let folder=std::env::temp_dir().join(format!("fujifilm-rust-{}",uuid::Uuid::new_v4()));
        fs::create_dir_all(&folder).map_err(|e|e.to_string())?;
        let result=(||->Result<(),String>{
            let mut archive=zip::ZipArchive::new(std::io::Cursor::new(&bytes)).map_err(|e|e.to_string())?;
            if archive.len()!=FILES.len(){return Err("Số file trong ZIP không hợp lệ".into())}
            for i in 0..archive.len(){
                let mut entry=archive.by_index(i).map_err(|e|e.to_string())?;
                let name=entry.name().replace('\\',"/");
                if !FILES.contains(&name.as_str())||entry.size()>60*1024*1024{return Err(format!("File không hợp lệ trong ZIP: {name}"))}
                let out=folder.join(&name);
                if let Some(parent)=out.parent(){fs::create_dir_all(parent).map_err(|e|e.to_string())?;}
                let mut f=fs::File::create(out).map_err(|e|e.to_string())?;
                std::io::copy(&mut entry,&mut f).map_err(|e|e.to_string())?;
            }
            for file in FILES{if !folder.join(file).is_file(){return Err(format!("Thiếu file {file}"))}}
            let script=folder.join("install-rust-update.ps1");
            fs::write(&script,include_str!("../../scripts/install-rust-update.ps1")).map_err(|e|format!("Không thể tạo installer: {e}"))?;
            // Tiến trình updater thoát trước khi PowerShell thay EXE.
            Command::new("powershell.exe").args(["-NoProfile","-NonInteractive","-ExecutionPolicy","Bypass","-WindowStyle","Hidden","-File"])
                .arg(script).arg("-Stage").arg(&folder).arg("-Target").arg(&dir)
                .arg("-TrayPid").arg(tray_pid.to_string()).arg("-UpdaterPid").arg(std::process::id().to_string())
                .creation_flags(0x08000000).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null())
                .spawn().map_err(|e|e.to_string())?;
            log(&format!("Đã kiểm tra SHA-256. Chuẩn bị cập nhật Rust từ {:?} lên {tag}",current));
            Ok(())
        })();
        if result.is_err(){let _=fs::remove_dir_all(folder);}
        result.map(|_|true)
    }
    use std::os::windows::process::CommandExt;
    pub fn main(){
        let args:Vec<String>=std::env::args().collect();
        let mut pid=0u32;
        if let Some(i)=args.iter().position(|x|x=="--tray-pid"){pid=args.get(i+1).and_then(|x|x.parse().ok()).unwrap_or(0);}
        let watch=args.iter().any(|x|x=="--watch");
        loop{
            if watch && pid!=0&&!parent_running(pid){break}
            match check(pid){Ok(true)=>break,Ok(false)=>(),Err(e)=>log(&format!("Kiểm tra update lỗi: {e}"))}
            if !watch{break}
            for _ in 0..360{thread::sleep(Duration::from_secs(60));if pid!=0&&!parent_running(pid){return}}
        }
    }
}

