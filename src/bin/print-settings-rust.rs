#![cfg_attr(windows, windows_subsystem = "windows")]
#[path = "../production/mod.rs"] mod production;

#[cfg(windows)]
fn installed_printers() -> Vec<String> {
    use std::process::Command;
    let script = "[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new(); Add-Type -AssemblyName System.Drawing; [System.Drawing.Printing.PrinterSettings]::InstalledPrinters | ForEach-Object { [Console]::WriteLine($_) }";
    let output = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", script]).output();
    match output {
        Ok(o) if o.status.success() => {
            let mut v: Vec<String> = String::from_utf8_lossy(&o.stdout).lines()
                .map(|s|s.trim().to_string()).filter(|s|!s.is_empty()).collect();
            v.sort(); v.dedup(); v
        }
        _ => Vec::new(),
    }
}

#[cfg(windows)]
fn pick_file(kind: &str) -> Option<String> {
    use std::process::Command;
    let code = match kind {
        "xlsx" => "Add-Type -AssemblyName System.Windows.Forms; $d=New-Object System.Windows.Forms.OpenFileDialog; $d.Filter='Excel (*.xlsx)|*.xlsx'; if($d.ShowDialog() -eq 'OK'){[Console]::Write($d.FileName)}",
        "exe" => "Add-Type -AssemblyName System.Windows.Forms; $d=New-Object System.Windows.Forms.OpenFileDialog; $d.Filter='Executable (*.exe)|*.exe'; if($d.ShowDialog() -eq 'OK'){[Console]::Write($d.FileName)}",
        _ => "Add-Type -AssemblyName System.Windows.Forms; $d=New-Object System.Windows.Forms.FolderBrowserDialog; if($d.ShowDialog() -eq 'OK'){[Console]::Write($d.SelectedPath)}",
    };
    let script = format!("[Console]::OutputEncoding=[System.Text.UTF8Encoding]::new(); {code}");
    let result = Command::new("powershell.exe").args(["-NoProfile","-STA","-Command",&script]).output().ok()?;
    if result.status.success() {
        let path=String::from_utf8_lossy(&result.stdout).trim().to_string();
        if !path.is_empty() {return Some(path)}
    }
    None
}

#[cfg(windows)]
fn startup(enabled: bool) -> Result<(),String> {
    use std::process::Command;
    let exe=production::Config::base_dir().join("Print-Agent.exe");
    if enabled && !exe.is_file(){return Err("Thiếu Print-Agent.exe trong thư mục ứng dụng".into())}
    let key=r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
    let status=if enabled {
        Command::new("reg.exe").args(["add",key,"/v","FujifilmPrintRust","/t","REG_SZ","/d",&format!("\"{}\"",exe.display()),"/f"]).status()
    }else{
        Command::new("reg.exe").args(["delete",key,"/v","FujifilmPrintRust","/f"]).status()
    };
    match status{Ok(s) if s.success()=>Ok(()),Ok(_) if !enabled=>Ok(()),Ok(s)=>Err(format!("Không lưu được Startup: {:?}",s.code())),Err(e)=>Err(e.to_string())}
}

#[cfg(windows)]
fn startup_checked() -> bool {
    std::process::Command::new("reg.exe").args(["query",r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run","/v","FujifilmPrintRust"])
        .output().map(|o|o.status.success()).unwrap_or(false)
}

#[cfg(windows)]
fn main() {
    use native_windows_gui as nwg;
    use production::Config;
    use std::{rc::Rc, cell::RefCell};

    nwg::init().expect("Không thể mở Print Settings");
    nwg::Font::set_global_family("Segoe UI").unwrap();

    let mut window=nwg::Window::default();
    nwg::Window::builder().title("Fujifilm Print - Cài đặt máy in / 印刷設定 (Rust)")
        .size((870,745)).position((150,60)).build(&mut window).unwrap();
    let c=Config::load(&Config::base_dir().join("config.ini")).unwrap_or_default();

    let fields=[
        ("API Machine-Control",c.api_base_url.clone()),
        ("Agent Token",c.agent_token.clone()),
        ("Tên hiển thị trên web / Web表示名",c.printer_display_name.clone()),
        ("File mẫu XLSX / テンプレート",c.template_path.clone()),
        ("Chu kỳ kiểm tra (giây) / 確認間隔",c.poll_seconds.to_string()),
        ("Thư mục PDF / PDF保存先",c.pdf_output_path.clone()),
        ("SumatraPDF.exe / PDF印刷ツール",c.pdf_printer_exe.clone()),
        ("Số trang tối đa mỗi PDF / 最大ページ数",c.max_pdf_pages.to_string()),
    ];
    let mut labels=Vec::new();let mut inputs=Vec::new();
    let positions=[12,58,150,196,242,334,380,426];
    for (i,(label,value)) in fields.iter().enumerate(){
        let y=positions[i];
        let mut l=nwg::Label::default();
        nwg::Label::builder().text(label).parent(&window).position((20,y)).size((320,28)).build(&mut l).unwrap();
        let mut t=nwg::TextInput::default();
        nwg::TextInput::builder().text(value).parent(&window).position((350,y)).size((380,28)).build(&mut t).unwrap();
        labels.push(l);inputs.push(t);
    }
    let mut p_label=nwg::Label::default();
    nwg::Label::builder().text("Máy in Windows / Windowsプリンター").parent(&window).position((20,104)).size((320,28)).build(&mut p_label).unwrap();
    let mut printers=vec!["(Máy in mặc định Windows)".to_string()];
    printers.extend(installed_printers());
    if !c.printer_name.is_empty()&&!printers.contains(&c.printer_name){printers.push(c.printer_name.clone());}
    let printer_index=printers.iter().position(|s|s==&c.printer_name).unwrap_or(0);
    let mut printer=nwg::ComboBox::<String>::default();
    nwg::ComboBox::builder().collection(printers).selected_index(Some(printer_index))
       .parent(&window).position((350,104)).size((380,180)).build(&mut printer).unwrap();

    let mut mode_label=nwg::Label::default();
    nwg::Label::builder().text("Chế độ in / 印刷方式").parent(&window).position((20,288)).size((320,28)).build(&mut mode_label).unwrap();
    let mut mode=nwg::ComboBox::<String>::default();
    nwg::ComboBox::builder()
      .collection(vec!["Xuất PDF rồi in / PDFを印刷".to_string(),"Chỉ lưu PDF / PDF保存のみ".to_string()])
      .selected_index(Some(if c.print_mode=="pdf"{1}else{0}))
      .parent(&window).position((350,288)).size((380,100)).build(&mut mode).unwrap();

    let mut browse_xlsx=nwg::Button::default();let mut browse_folder=nwg::Button::default();let mut browse_exe=nwg::Button::default();
    let mut refresh=nwg::Button::default();
    nwg::Button::builder().text("Làm mới / 更新").parent(&window).position((741,104)).size((108,28)).build(&mut refresh).unwrap();
    nwg::Button::builder().text("Chọn file...").parent(&window).position((741,196)).size((108,28)).build(&mut browse_xlsx).unwrap();
    nwg::Button::builder().text("Chọn thư mục...").parent(&window).position((741,334)).size((108,28)).build(&mut browse_folder).unwrap();
    nwg::Button::builder().text("Chọn EXE...").parent(&window).position((741,380)).size((108,28)).build(&mut browse_exe).unwrap();

    let mut auto=nwg::CheckBox::default(); let mut boot=nwg::CheckBox::default();
    nwg::CheckBox::builder().text("Tự động kiểm tra cập nhật khi rảnh / 待機中に自動更新").parent(&window)
        .position((20,486)).size((550,32))
        .check_state(if c.auto_update_enabled{nwg::CheckBoxState::Checked}else{nwg::CheckBoxState::Unchecked})
        .build(&mut auto).unwrap();
    nwg::CheckBox::builder().text("Khởi động cùng Windows khi đăng nhập / Windows起動").parent(&window)
        .position((20,527)).size((550,32))
        .check_state(if startup_checked(){nwg::CheckBoxState::Checked}else{nwg::CheckBoxState::Unchecked})
        .build(&mut boot).unwrap();

    let mut note=nwg::Label::default();
    nwg::Label::builder().text("Lưu cấu hình: config.ini. Chỉ thử in khi đã kiểm tra mẫu Excel 2013 và máy in.")
        .parent(&window).position((20,579)).size((820,28)).build(&mut note).unwrap();
    let mut save=nwg::Button::default();let mut close=nwg::Button::default();
    nwg::Button::builder().text("Lưu cấu hình / 設定を保存").parent(&window).position((465,633)).size((205,42)).build(&mut save).unwrap();
    nwg::Button::builder().text("Đóng / 閉じる").parent(&window).position((692,633)).size((150,42)).build(&mut close).unwrap();

    let inputs=Rc::new(inputs);
    let p=Rc::new(printer);let m=Rc::new(mode);
    let a=Rc::new(auto);let b=Rc::new(boot);
    let handles=(save.handle,close.handle,refresh.handle,browse_xlsx.handle,browse_folder.handle,browse_exe.handle);
    let win=window.handle;
    let callback=nwg::full_bind_event_handler(&win,move|ev,_data,h|{
        if ev==nwg::Event::OnWindowClose&&h==win {nwg::stop_thread_dispatch();return}
        if ev!=nwg::Event::OnButtonClick {return}
        if h==handles.1 {nwg::stop_thread_dispatch();return}
        if h==handles.2 {
            // Đóng/mở lại GUI để tạo lại danh sách đã lấy từ Windows Spooler.
            if let Ok(exe)=std::env::current_exe(){
                let _=std::process::Command::new(exe).spawn();
                nwg::stop_thread_dispatch();
            }
        } else if h==handles.3 {if let Some(s)=pick_file("xlsx"){inputs[3].set_text(&s);}}
        else if h==handles.4 {if let Some(s)=pick_file("folder"){inputs[5].set_text(&s);}}
        else if h==handles.5 {if let Some(s)=pick_file("exe"){inputs[6].set_text(&s);}}
        else if h==handles.0 {
            let parse=||->Result<Config,String>{
                let poll_seconds=inputs[4].text().parse::<u64>().map_err(|_|"Chu kỳ phải là số")?;
                let max_pdf_pages=inputs[7].text().parse::<usize>().map_err(|_|"Số trang phải là số")?;
                let printer_name=if p.selection_string().as_deref()==Some("(Máy in mặc định Windows)"){String::new()}else{p.selection_string().unwrap_or_default()};
                let c=Config{
                    api_base_url:inputs[0].text().trim().into(),agent_token:inputs[1].text().trim().into(),
                    printer_name,printer_display_name:inputs[2].text().trim().into(),
                    template_path:inputs[3].text().trim().into(),poll_seconds,max_pdf_pages,
                    pdf_output_path:inputs[5].text().trim().into(),pdf_printer_exe:inputs[6].text().trim().into(),
                    print_mode:if m.selection_string().as_deref()==Some("Chỉ lưu PDF / PDF保存のみ"){"pdf".into()}else{"pdfprint".into()},
                    auto_update_enabled:a.check_state()==nwg::CheckBoxState::Checked,
                };
                c.check()?;
                if !c.template_path.to_ascii_lowercase().ends_with(".xlsx"){return Err("Mẫu phải là file .xlsx".into())}
                if !c.pdf_output_path.is_empty()&&!std::path::Path::new(&c.pdf_output_path).is_absolute(){return Err("Thư mục PDF phải là đường dẫn tuyệt đối".into())}
                if c.print_mode=="pdfprint"&&!c.pdf_printer_exe.to_ascii_lowercase().ends_with(".exe"){return Err("Bộ in PDF cần file .exe".into())}
                Ok(c)
            };
            let result=parse().and_then(|c|c.save(&Config::base_dir().join("config.ini")))
                .and_then(|_|startup(b.check_state()==nwg::CheckBoxState::Checked));
            match result{
                Ok(())=>{nwg::simple_message("Fujifilm Print","Đã lưu config.ini và cài đặt khởi động cùng Windows. Hãy khởi động lại Agent.");}
                Err(e)=>{nwg::error_message("Không lưu được cấu hình",&e);}
            }
        }
    });
    nwg::dispatch_thread_events();
    nwg::unbind_event_handler(&callback);
}
#[cfg(not(windows))]
fn main(){eprintln!("Print Settings Rust chỉ chạy trên Windows");}
