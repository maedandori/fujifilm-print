#![cfg_attr(windows, windows_subsystem = "windows")]
#[cfg(windows)]
fn main() {
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
 nwg::MenuItem::builder().text("Bắt đầu nhận lệnh (in thật) / 開始").parent(&menu).build(&mut start).unwrap();
 nwg::MenuItem::builder().text("Dừng sau job hiện tại / 停止").parent(&menu).build(&mut stop).unwrap();
 nwg::MenuItem::builder().text("Print Settings / 設定").parent(&menu).build(&mut settings).unwrap();
 nwg::MenuItem::builder().text("Xem nhật ký / ログ").parent(&menu).build(&mut log).unwrap();
 nwg::MenuItem::builder().text("Trạng thái / 状態").parent(&menu).build(&mut status).unwrap();
 nwg::MenuItem::builder().text("Kiểm tra cập nhật Rust / 更新確認").parent(&menu).build(&mut update).unwrap();
 nwg::MenuItem::builder().text("Thoát giao diện / 終了").parent(&menu).build(&mut exit).unwrap();
 let base=std::env::current_exe().unwrap().parent().unwrap().to_path_buf();
 // Rust updater kiểm tra bản phát hành riêng mỗi 6 giờ khi được bật trong config.ini.
 let updater=base.join("Print-Updater-Rust.exe");
 if updater.is_file() && crate::enabled_updates(&base){
   let _=Command::new(&updater).arg("--watch").arg("--tray-pid").arg(std::process::id().to_string())
     .creation_flags(0x08000000).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn();
 }
 let process:Rc<RefCell<Option<Child>>>=Rc::new(RefCell::new(None));
 let child=Rc::clone(&process);
 let handles=(tray.handle,start.handle,stop.handle,settings.handle,log.handle,status.handle,exit.handle,update.handle);
 let menu=Rc::new(menu);let m=Rc::clone(&menu);let win=window.handle;
 let callback=nwg::full_bind_event_handler(&win,move|ev,_data,h|{
  if ev==nwg::Event::OnContextMenu && h==handles.0 {let (x,y)=nwg::GlobalCursor::position();m.popup(x,y);return}
  if ev!=nwg::Event::OnMenuItemSelected {return}
  if h==handles.1 {
   let mut guard=child.borrow_mut();
   if let Some(p)=guard.as_mut(){if p.try_wait().ok().flatten().is_none(){nwg::simple_message("Fujifilm Rust","Agent đang hoạt động");return}}
   let exe=base.join("Print-Agent-Rust-Core.exe");
   if !exe.exists(){nwg::error_message("Fujifilm Rust","Không tìm thấy Print-Agent-Rust.exe");return}
   let msg="CHÚ Ý: --enable-live sẽ nhận job thật từ Cloudflare và có thể IN GIẤY. Chỉ thử trên máy an toàn. Tiếp tục?";
   if nwg::message(&nwg::MessageParams{title:"Bật in thật?",content:msg,buttons:nwg::MessageButtons::YesNo,icons:nwg::MessageIcons::Warning})!=nwg::MessageChoice::Yes{return}
   let _=std::fs::remove_file(base.join("rust-stop-request.flag"));
   match Command::new(exe).arg("--enable-live").creation_flags(0x08000000).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn(){
    Ok(proc)=>{*guard=Some(proc);nwg::simple_message("Fujifilm Rust","Đã bắt đầu nhận lệnh. Xem log để kiểm tra trạng thái.");}
    Err(e)=>{nwg::error_message("Lỗi khởi động Agent",&e.to_string());}
   }
  }else if h==handles.2{
   let _=std::fs::write(base.join("rust-stop-request.flag"),"stop");
   nwg::simple_message("Fujifilm Rust","Đã yêu cầu dừng sau khi hoàn thành job hiện tại.");
  }else if h==handles.3 {
   let exe=base.join("Print-Settings-Rust.exe");if let Err(e)=Command::new(exe).spawn(){nwg::error_message("Không mở được Settings",&e.to_string());}
  }else if h==handles.4 {
   let path=base.join("print-agent-rust.log");
   if let Err(e)=Command::new("notepad.exe").arg(path).spawn(){nwg::error_message("Không mở được nhật ký",&e.to_string());}
  }else if h==handles.5 {
   let mut guard=child.borrow_mut();
   let running=guard.as_mut().and_then(|c|c.try_wait().ok().flatten()).is_some_and(|status|!status.success());
   let text=if guard.as_mut().and_then(|c|c.try_wait().ok()).flatten().is_none() && guard.is_some(){"Agent Rust có tiến trình đang chạy"}else if running{"Agent Rust đã dừng với lỗi"}else{"Agent Rust chưa chạy / đã thoát"};
   nwg::simple_message("Trạng thái Fujifilm Rust",text);
  }else if h==handles.7 {
   let exe=base.join("Print-Updater-Rust.exe");
   if let Err(e)=Command::new(exe).arg("--tray-pid").arg(std::process::id().to_string())
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
