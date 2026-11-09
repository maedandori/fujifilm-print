#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(windows)]
fn main() {
    use native_windows_gui as nwg;
    use std::{cell::RefCell, rc::Rc, time::Instant};

    nwg::init().expect("Không thể khởi tạo giao diện Windows");
    nwg::Font::set_global_family("Segoe UI").expect("Không thể tải font");

    let mut window = nwg::Window::default();
    let mut title = nwg::Label::default();
    let mut description = nwg::Label::default();
    let mut status = nwg::Label::default();
    let mut validate = nwg::Button::default();
    let mut benchmark = nwg::Button::default();
    let mut close = nwg::Button::default();

    nwg::Window::builder()
        .size((560, 300))
        .position((240, 180))
        .title("Fujifilm Print - Rust Test v0.2.0")
        .build(&mut window).expect("Không thể tạo cửa sổ");

    nwg::Label::builder().text("FUJIFILM PRINT - RUST").parent(&window)
        .position((24, 18)).size((500, 32)).build(&mut title).unwrap();
    nwg::Label::builder().text("Bản thử nghiệm an toàn: không kết nối Cloudflare, không gửi lệnh in.")
        .parent(&window).position((24, 55)).size((515, 28))
        .build(&mut description).unwrap();
    nwg::Label::builder().text("Trạng thái: Sẵn sàng. Cửa sổ sẽ ở lại cho đến khi bạn đóng.")
        .parent(&window).position((24, 102)).size((510, 75))
        .build(&mut status).unwrap();

    nwg::Button::builder().text("Kiểm tra JSON mẫu").parent(&window)
        .position((24, 202)).size((157, 42)).build(&mut validate).unwrap();
    nwg::Button::builder().text("Đo tốc độ Rust").parent(&window)
        .position((195, 202)).size((157, 42)).build(&mut benchmark).unwrap();
    nwg::Button::builder().text("Đóng").parent(&window)
        .position((367, 202)).size((157, 42)).build(&mut close).unwrap();

    let handle = window.handle;
    let validate_handle = validate.handle;
    let benchmark_handle = benchmark.handle;
    let close_handle = close.handle;
    let status = Rc::new(RefCell::new(status));
    let status_ref = Rc::clone(&status);
    let handler = nwg::full_bind_event_handler(&handle, move |event, _data, control| {
        use nwg::Event as E;
        match event {
            E::OnWindowClose if control == handle => nwg::stop_thread_dispatch(),
            E::OnButtonClick if control == close_handle => nwg::stop_thread_dispatch(),
            E::OnButtonClick if control == validate_handle => {
                let sample = include_str!("../../sample-job.json");
                let result = serde_json::from_str::<serde_json::Value>(sample);
                let text = match result {
                    Ok(v) => format!("JSON mẫu hợp lệ: {} thẻ, {} bản sao. KHÔNG IN.",
                        v["cards"].as_array().map_or(0, |a| a.len()),
                        v["copies"].as_u64().unwrap_or(0)),
                    Err(e) => format!("Lỗi JSON: {e}"),
                };
                status_ref.borrow().set_text(&text);
            }
            E::OnButtonClick if control == benchmark_handle => {
                let sample = include_str!("../../sample-job.json");
                let start = Instant::now();
                let mut count = 0usize;
                for _ in 0..100_000 {
                    match serde_json::from_str::<serde_json::Value>(sample) {
                        Ok(v) => count += v["cards"].as_array().map_or(0, |a| a.len()),
                        Err(e) => {
                            status_ref.borrow().set_text(&format!("Benchmark lỗi: {e}"));
                            return;
                        }
                    }
                }
                status_ref.borrow().set_text(&format!(
                    "100.000 lượt đọc JSON: {:.2} ms / {} thẻ đã đọc. KHÔNG IN.",
                    start.elapsed().as_secs_f64() * 1000.0, count
                ));
            }
            _ => {}
        }
    });
    nwg::dispatch_thread_events();
    nwg::unbind_event_handler(&handler);
}

#[cfg(not(windows))]
fn main() {
    eprintln!("Giao diện này chỉ hỗ trợ Windows.");
}
