# Fujifilm Print — Rust thử nghiệm

Đây là bản thử nghiệm Rust **độc lập** để so sánh với `fujiserox-print` C#/.NET 8.

**AN TOÀN:** Không kết nối Cloudflare production, không gửi lệnh tới máy in, không thay đổi Print Agent hiện tại. Chưa thay thế bản C# tại công ty.

## Chạy

Cần Rust stable trên Windows 10/11 x64:

```powershell
cargo test --release
cargo run --release -- --benchmark 100000
cargo run --release -- --simulate sample-job.json
cargo run --release -- --idle-seconds 30
```

Để nhận EXE không cần cài Rust trên máy, mở **Actions > Rust Print Agent (thử nghiệm)**, tải artifact `Rust-Print-Agent-Windows-x64` và giải nén.

## Chức năng phiên bản thử nghiệm

- Nhận và xác thực cấu trúc JSON mô phỏng lệnh in; chỉ xác thực, **không in**.
- Benchmark xử lý lệnh in giả lập (không đo driver, PDF hoặc tốc độ máy in).
- Chế độ chạy nền giả lập để đo RAM trên Windows.
- GitHub Actions tự kiểm thử, build bản Release và ghi kích thước EXE/RAM mẫu.

## Trạng thái

Đây là **prototype**, chưa có WebSocket, SumatraPDF, INI, tray icon, cập nhật tự động hay tích hợp Cloudflare. Chưa được xác nhận tương thích với JSON/giao thức của Print Agent C# thực tế.

## So sánh công bằng

Nên đo C# và Rust trên **cùng Windows**, cùng điều kiện idle, cùng số lượng lệnh và cùng cách đóng gói. Số đo Rust trong CI không phải benchmark trực tiếp với C#.
