#![cfg_attr(windows, windows_subsystem = "windows")]
#[path = "../production/mod.rs"] mod production;

#[cfg(windows)]
fn main(){
 use native_windows_gui as nwg;
 use production::Config;
 use std::rc::Rc;
 nwg::init().expect("Không thể mở giao diện Windows");
 nwg::Font::set_global_family("Segoe UI").unwrap();
 let mut window=nwg::Window::default();
 nwg::Window::builder().title("Fujifilm Print Settings Rust v0.3 (Thử nghiệm)")
  .size((760,670)).position((180,70)).build(&mut window).unwrap();
 let names=[
  ("Cloudflare API URL / API URL","ApiBaseUrl"),
  ("Agent Token / トークン","AgentToken"),
  ("Windows Printer / プリンター","PrinterName"),
  ("Tên hiển thị trên web / 表示名","PrinterDisplayName"),
  ("Đường dẫn mẫu XLSX / テンプレート","TemplatePath"),
  ("Thư mục PDF / PDF出力先","PdfOutputPath"),
  ("SumatraPDF.exe / 印刷ツール","PdfPrinterExe"),
  ("Chế độ pdf hoặc pdfprint / 印刷方式","PrintMode"),
  ("Chu kỳ nhận lệnh (giây) / 取得間隔","PollSeconds"),
  ("Số trang PDF tối đa / 最大ページ数","MaxPdfPages"),
  ("Tự cập nhật 1/0 / 自動更新","AutoUpdateEnabled"),
 ];
 let c=Config::load(&Config::base_dir().join("config.ini")).unwrap_or_default();
 let vals=vec![c.api_base_url,c.agent_token,c.printer_name,c.printer_display_name,c.template_path,c.pdf_output_path,c.pdf_printer_exe,c.print_mode,c.poll_seconds.to_string(),c.max_pdf_pages.to_string(),if c.auto_update_enabled{"1".into()}else{"0".into()}];
 let mut entries=vec![];
 let mut labels=vec![];
 for (i,(label,_)) in names.iter().enumerate(){
  let y=14+(i as i32)*48;
  let mut l=nwg::Label::default();nwg::Label::builder().text(label).parent(&window).position((20,y)).size((310,26)).build(&mut l).unwrap();labels.push(l);
  let mut t=nwg::TextInput::default();nwg::TextInput::builder().text(&vals[i]).parent(&window).position((340,y)).size((380,28)).build(&mut t).unwrap();entries.push(t);
 }
 let mut save=nwg::Button::default();let mut exit=nwg::Button::default();let mut info=nwg::Label::default();
 nwg::Button::builder().text("Lưu cấu hình / 保存").parent(&window).position((340,555)).size((180,42)).build(&mut save).unwrap();
 nwg::Button::builder().text("Đóng / 閉じる").parent(&window).position((540,555)).size((180,42)).build(&mut exit).unwrap();
 nwg::Label::builder().text("Thử nghiệm: giữ nguyên config.ini và agent-id.txt khi nâng cấp.").parent(&window).position((20,610)).size((700,30)).build(&mut info).unwrap();
 let entries=Rc::new(entries);let e=Rc::clone(&entries);
 let save_handle=save.handle;let exit_handle=exit.handle;let win_handle=window.handle;
 let handler=nwg::full_bind_event_handler(&win_handle,move|event,_data,control|{
  match event{
   nwg::Event::OnWindowClose if control==win_handle=>nwg::stop_thread_dispatch(),
   nwg::Event::OnButtonClick if control==exit_handle=>nwg::stop_thread_dispatch(),
   nwg::Event::OnButtonClick if control==save_handle=>{
    let get=|n:usize|e[n].text();
    let seconds=get(8).parse::<u64>();
    let pages=get(9).parse::<usize>();
    let result=match(seconds,pages){(Ok(poll_seconds),Ok(max_pdf_pages))=>{
     let c=Config{api_base_url:get(0),agent_token:get(1),printer_name:get(2),printer_display_name:get(3),template_path:get(4),
      pdf_output_path:get(5),pdf_printer_exe:get(6),print_mode:get(7),poll_seconds,max_pdf_pages,auto_update_enabled:get(10)!="0"};
     c.save(&Config::base_dir().join("config.ini"))
    },_=>Err("PollSeconds hoặc MaxPdfPages không hợp lệ".into())};
    match result{Ok(())=>nwg::simple_message("Fujifilm Print","Đã lưu config.ini / 保存しました"),Err(err)=>nwg::error_message("Không thể lưu / 保存失敗",&err)}
   }
   _=>{}
  }
 });
 nwg::dispatch_thread_events();
 nwg::unbind_event_handler(&handler);
}
#[cfg(not(windows))]
fn main(){eprintln!("Print Settings hỗ trợ Windows.");}
