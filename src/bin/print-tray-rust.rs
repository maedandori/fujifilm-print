#![cfg_attr(windows, windows_subsystem = "windows")]
#[path = "../production/mod.rs"] mod production;
#[path = "../production/update.rs"] mod update;
#[cfg(windows)]
fn tray_main() {
 use native_windows_gui as nwg;
 use std::{cell::RefCell,rc::Rc,process::{Command,Child,Stdio},os::windows::process::CommandExt};
 nwg::init().expect("Không thể khởi tạo Windows Tray");
 let mut window=nwg::MessageWindow::default();
 let mut tray=nwg::TrayNotification::default();
 let icon=nwg::Icon::from_system(nwg::OemIcon::Information);
 nwg::MessageWindow::builder().build(&mut window).unwrap();
 nwg::TrayNotification::builder().parent(&window).icon(Some(&icon)).tip(Some("Fujifilm Print Rust - Thử nghiệm")).build(&mut tray).unwrap();
 let mut menu=nwg::Menu::default();
 nwg::Menu::builder().popup(true).parent(&window).build(&mut menu).unwrap();
 let mut start=nwg::MenuItem::default();
 let mut stop=nwg::MenuItem::default();
 let mut settings=nwg::MenuItem::default();
 let mut log=nwg::MenuItem::default();
 let mut status=nwg::MenuItem::default();
 let mut exit=nwg::MenuItem::default();
 let mut update=nwg::MenuItem::default();
 let mut diagnostic=nwg::MenuItem::default();
 nwg::MenuItem::builder().text("Bắt đầu nhận lệnh (in thật) / 開始").parent(&menu).build(&mut start).unwrap();
 nwg::MenuItem::builder().text("Dừng sau job hiện tại / 停止").parent(&menu).build(&mut stop).unwrap();
 nwg::MenuItem::builder().text("Print Settings / 設定").parent(&menu).build(&mut settings).unwrap();
 nwg::MenuItem::builder().text("Xem nhật ký / ログ").parent(&menu).build(&mut log).unwrap();
 nwg::MenuItem::builder().text("Trạng thái / 状態").parent(&menu).build(&mut status).unwrap();
 nwg::MenuItem::builder().text("Kiểm tra cấu hình / 設定確認").parent(&menu).build(&mut diagnostic).unwrap();
 nwg::MenuItem::builder().text("Kiểm tra cập nhật Rust / 更新確認").parent(&menu).build(&mut update).unwrap();
 nwg::MenuItem::builder().text("Thoát giao diện / 終了").parent(&menu).build(&mut exit).unwrap();
 let base=std::env::current_exe().unwrap().parent().unwrap().to_path_buf();
 // Rust updater kiểm tra bản phát hành riêng mỗi 6 giờ khi được bật trong config.ini.
 let updater=std::env::current_exe().unwrap();
 if crate::enabled_updates(&base){
   let _=Command::new(&updater).arg("--update-check").arg("--watch").arg("--tray-pid").arg(std::process::id().to_string())
     .creation_flags(0x08000000).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn();
 }
 let process:Rc<RefCell<Option<Child>>>=Rc::new(RefCell::new(None));
 let child=Rc::clone(&process);
 let handles=(tray.handle,start.handle,stop.handle,settings.handle,log.handle,status.handle,exit.handle,update.handle,diagnostic.handle);
 let menu=Rc::new(menu);let m=Rc::clone(&menu);let win=window.handle;
 let callback=nwg::full_bind_event_handler(&win,move|ev,_data,h|{
  if ev==nwg::Event::OnContextMenu && h==handles.0 {let (x,y)=nwg::GlobalCursor::position();m.popup(x,y);return}
  if ev!=nwg::Event::OnMenuItemSelected {return}
  if h==handles.1 {
   let mut guard=child.borrow_mut();
   if let Some(p)=guard.as_mut(){if p.try_wait().ok().flatten().is_none(){nwg::simple_message("Fujifilm Rust","Agent đang hoạt động");return}}
   let exe=std::env::current_exe().unwrap();
   if !exe.exists(){nwg::error_message("Fujifilm Rust","Không tìm thấy Print-Agent.exe.");return}
   if let Err(e)=check_local_config(&base){nwg::error_message("Chưa thể khởi động Agent",&e);return}
   let msg="CHÚ Ý: --enable-live sẽ nhận job thật từ Cloudflare và có thể IN GIẤY. Chỉ thử trên máy an toàn. Tiếp tục?";
   if nwg::message(&nwg::MessageParams{title:"Bật in thật?",content:msg,buttons:nwg::MessageButtons::YesNo,icons:nwg::MessageIcons::Warning})!=nwg::MessageChoice::Yes{return}
   let _=std::fs::remove_file(base.join("rust-stop-request.flag"));
   match Command::new(exe).args(["--core","--enable-live"]).creation_flags(0x08000000).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn(){
    Ok(mut proc)=>{
      std::thread::sleep(std::time::Duration::from_millis(350));
      match proc.try_wait(){
       Ok(Some(code))=>{
         *guard=Some(proc);
         nwg::error_message("Agent đã thoát khi khởi động",&format!("Mã thoát: {:?}. Mở Xem nhật ký để biết nguyên nhân. Kiểm tra config.ini và Agent Token.",code.code()));
       }
       Ok(None)=>{*guard=Some(proc);nwg::simple_message("Fujifilm Rust","Tiến trình Agent đã khởi chạy. Kết nối thực tế xem trong print-agent-rust.log.");}
       Err(e)=>{*guard=Some(proc);nwg::error_message("Không kiểm tra được tiến trình",&e.to_string());}
      }
    }
    Err(e)=>{nwg::error_message("Lỗi khởi động Agent",&e.to_string());}
   }
  }else if h==handles.2{
   let _=std::fs::write(base.join("rust-stop-request.flag"),"stop");
   nwg::simple_message("Fujifilm Rust","Đã yêu cầu dừng sau khi hoàn thành job hiện tại.");
  }else if h==handles.3 {
   let exe=base.join("Print-Settings.exe");if let Err(e)=Command::new(exe).spawn(){nwg::error_message("Không mở được Settings",&e.to_string());}
  }else if h==handles.4 {
   let path=base.join("print-agent-rust.log");
   if let Err(e)=Command::new("notepad.exe").arg(path).spawn(){nwg::error_message("Không mở được nhật ký",&e.to_string());}
  }else if h==handles.5 {
   let mut guard=child.borrow_mut();
   let message=match guard.as_mut(){
     None=>"Agent chưa bật nhận lệnh. Đây không phải lỗi. Nhấp phải icon rồi chọn 'Bắt đầu nhận lệnh (in thật)' khi đã cấu hình và sẵn sàng thử in.".to_string(),
     Some(proc)=>match proc.try_wait(){
       Ok(None)=>"Agent đang chạy. Để xác định kết nối Cloudflare, mở nhật ký print-agent-rust.log.".to_string(),
       Ok(Some(exit))=>format!("Agent đã thoát. Mã thoát: {:?}. Mở nhật ký để xem nguyên nhân; kiểm tra config.ini và AgentToken.",exit.code()),
       Err(e)=>format!("Lỗi khi kiểm tra tiến trình: {e}")
     }
   };
   nwg::simple_message("Trạng thái Fujifilm Rust",&message);
  }else if h==handles.8 {
   match check_local_config(&base){
     Ok(())=>{nwg::simple_message("Kiểm tra cấu hình","Các mục cốt lõi của config.ini hợp lệ. Chưa kiểm tra xác thực Cloudflare hoặc máy in.");},
     Err(e)=>{nwg::error_message("Cấu hình chưa sẵn sàng",&e);}
   };
  }else if h==handles.7 {
   let exe=std::env::current_exe().unwrap();
   if let Err(e)=Command::new(exe).arg("--update-check").arg("--tray-pid").arg(std::process::id().to_string())
      .creation_flags(0x08000000).spawn(){nwg::error_message("Không kiểm tra được cập nhật",&e.to_string());}
   else{nwg::simple_message("Fujifilm Rust","Đã yêu cầu kiểm tra GitHub Release Rust; kết quả ghi trong rust-update.log.");}
  }else if h==handles.6 {
   let mut guard=child.borrow_mut();
   if let Some(c)=guard.as_mut(){if c.try_wait().ok().flatten().is_none(){
    nwg::simple_message("Chưa thể thoát","Agent đang nhận job. Hãy chọn 'Dừng sau job hiện tại', kiểm tra trạng thái rồi thoát.");return
   }}
   nwg::stop_thread_dispatch();
  }
 });
 nwg::dispatch_thread_events();
 nwg::unbind_event_handler(&callback);
}
#[cfg(not(windows))]
fn main(){eprintln!("System Tray chỉ hỗ trợ Windows");}

#[cfg(windows)]
fn enabled_updates(base:&std::path::Path)->bool{
  std::fs::read_to_string(base.join("config.ini")).ok().map(|v|!v.lines().any(|l|l.trim().eq_ignore_ascii_case("AutoUpdateEnabled=0"))).unwrap_or(false)
}

#[cfg(windows)]
fn check_local_config(base:&std::path::Path)->Result<(),String>{
 let file=base.join("config.ini");
 let raw=std::fs::read_to_string(file).map_err(|_|"Chưa có config.ini. Hãy mở Print-Settings-Rust.exe và lưu cấu hình.".to_string())?;
 let get=|key:&str|->String{raw.lines().filter_map(|line|line.split_once('=')).find(|(k,_)|k.trim().eq_ignore_ascii_case(key)).map(|(_,v)|v.trim().to_string()).unwrap_or_default()};
 let url=get("ApiBaseUrl");
 if !(url.starts_with("https://")||url.starts_with("http://localhost:")||url.starts_with("http://127.0.0.1:")){return Err("ApiBaseUrl phải là HTTPS hợp lệ (hoặc HTTP localhost).".into())}
 if get("AgentToken").is_empty(){return Err("AgentToken đang trống. Hãy lưu token trong Print Settings.".into())}
 let template=get("TemplatePath");
 if template.is_empty(){return Err("Chưa chọn TemplatePath.".into())}
 let path=std::path::Path::new(&template);
 let template_path=if path.is_absolute(){path.to_path_buf()}else{base.join(path)};
 if !template_path.is_file(){return Err(format!("Không thấy mẫu XLSX: {}. Hãy chọn lại trong Print Settings.",template_path.display()))}
 // Script Excel PDF được nhúng bên trong EXE và tạo tự động khi in.
 Ok(())
}

#[cfg(windows)]
fn main(){
 let args:Vec<String>=std::env::args().collect();
 if args.iter().any(|v|v=="--update-check"){update::app::main();return;}
 if args.iter().any(|v|v=="--core"){
   if let Err(e)=production::engine::run(args.iter().any(|v|v=="--enable-live")){production::log(&format!("LỖI CORE: {e}"));std::process::exit(1);}
   return;
 }
 if args.iter().any(|v|v=="--test-pdf"){
   let result=(||->Result<(),String>{
     let c=production::Config::load(&production::Config::base_dir().join("config.ini"))?;
     let data=std::fs::read_to_string(production::Config::base_dir().join("sample-job.json")).map_err(|e|e.to_string())?;
     let j=production::Job::parse(&data)?;
     production::engine::pdf_output(&c,&j)
   })();
   if let Err(e)=result{production::log(&format!("LỖI PDF: {e}"));std::process::exit(1)}
   return;
 }
 tray_main();
}
