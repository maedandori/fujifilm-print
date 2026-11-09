# Fujifilm Print Rust — hai ứng dụng Windows

**Bản hiện tại chỉ đóng gói hai file EXE:**
- `Print-Agent.exe`: chạy System Tray, nhận lệnh qua chế độ `--core`, tự kiểm tra cập nhật qua chế độ `--update-check`; hỗ trợ `--test-pdf`.
- `Print-Settings.exe`: cấu hình Windows printer, chế độ `pdfprint` hoặc `pdf`, đường dẫn mẫu Excel, SumatraPDF, tự cập nhật và khởi động cùng Windows.

**Không còn cần:** `Print-Agent-Rust-Core.exe`, `Print-Updater-Rust.exe` hay `Print-Agent-Rust-Tray.exe`. Script Excel và cài update đã được nhúng trong Print-Agent.exe và có thể được tạo lúc cần. Các file cấu hình, template, PDF và log vẫn được tạo bên ngoài khi chạy.

GitHub Actions: [Build Fujifilm Print Rust - 2 EXE](https://github.com/maedandori/fujifilm-print/actions). Chỉ tải artifact có build thành công `Fujifilm-Print-Rust-Two-EXE-Windows`.

**Không dùng chung Agent ID với bản C# đang chạy; không cho hai Agent cùng nhận job trên máy công ty khi chưa kiểm thử.** Khi mở biểu tượng khay, Agent chưa tự nhận job; phải xác nhận thao tác bật. Chưa kiểm chứng Excel 2013, driver Fuji Xerox hoặc in giấy thực tế, và chưa kiểm thử cơ chế cập nhật qua hai phiên bản trên máy Windows thật.

---

# Fujifilm Print — Rust thử nghiệm

Đây là bản thử nghiệm Rust **độc lập** để so sánh với `fujiserox-print` C#/.NET 8.

**AN TOÀN:** Không kết nối Cloudflare production, không gửi lệnh tới máy in, không thay đổi Print Agent hiện tại. Chưa thay thế bản C# tại công ty.

## Giao diện Windows v0.2.0

Nhấp đúp **Fujifilm-Print-Rust-Test.exe** để mở cửa sổ. Có nút kiểm tra JSON mẫu, benchmark và đóng cửa sổ. File **Fujifilm-Print-Rust-CLI.exe** là bản dành cho lệnh benchmark qua PowerShell, mở trực tiếp sẽ thoát như phiên bản cũ. Giao diện chỉ chạy trên Windows, chưa in thật.

## Chạy

Cần Rust stable trên Windows 10/11 x64:

```powershell
cargo test --release
cargo run --release --bin fujifilm-print-gui
cargo run --release --bin fujifilm-print-rust -- --benchmark 100000
cargo run --release --bin fujifilm-print-rust -- --simulate sample-job.json
cargo run --release --bin fujifilm-print-rust -- --idle-seconds 30
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


## Bản chuyển đổi C# → Rust (đang thử nghiệm)

Bản Rust được chia thành các file:
- **Print-Agent-Rust-Tray.exe**: System Tray, mở Settings, xem log, bật nhận lệnh có xác nhận, yêu cầu dừng sau khi xử lý xong job
- **Print-Settings-Rust.exe**: sửa và lưu config.ini, đủ các trường tương thích C#
- **Print-Agent-Rust.exe**: Agent dòng lệnh; mặc định KHÔNG nhận job; chỉ khi chạy với `--enable-live` mới thực sự kết nối và xử lý
- **scripts/export-excel-pdf.ps1**: cần đặt trong thư mục `scripts` bên cạnh EXE; dùng Excel 2013 COM để xuất template.pdf

Các chức năng đã có mã nguồn: đọc INI và agent-id.txt, JSON cards/cells và isFinal(完), chia tối đa MaxPdfPages, Excel 原紙 → template.pdf, SumatraPDF, WebSocket đánh thức + polling, HTTP Bearer, heartbeat Windows Spooler, cập nhật cấu hình INI từ web, yêu cầu tải SumatraPDF từ Worker có kiểm tra SHA-256, báo status job, log, System Tray, Settings.

**Chưa tương đương C# hoàn toàn:** tự nâng cấp hai EXE Rust qua GitHub Releases/Worker (manifest production hiện dành cho C#, Rust chủ động không áp dụng), giao diện chọn file/máy in như WinForms cũ, tự bật theo Windows, UI đổi ngôn ngữ động, kiểm chứng trên Excel 2013/driver Fuji Xerox thật. Gói mới cũng chưa thay đổi bất kỳ Worker Cloudflare nào. Các phần này KHÔNG được coi là đã hoàn tất.

### Hướng dẫn kiểm thử không in
1. Tải artifact mới nhất ở Actions **chỉ khi build Success**. Giải nén **toàn bộ** ZIP vào thư mục riêng, không đè bản C#.
2. Mở `Print-Settings-Rust.exe` để tạo `config.ini`. Nếu copy config từ C#, **đừng dùng chung Agent ID khi Agent C# đang chạy**.
3. Chế độ xuất PDF nhưng không in: cấu hình `PrintMode=pdf`, template.xlsx và đường dẫn PDF phù hợp, mở PowerShell ở thư mục EXE và chạy `./Print-Agent-Rust.exe --test-pdf`.
4. Để mở System Tray, chạy `Print-Agent-Rust-Tray.exe`. Việc bấm **Bắt đầu nhận lệnh (in thật)** sẽ có hộp xác nhận rõ ràng.
5. Không chạy C# và Rust đồng thời cùng một `agent-id.txt`: có thể gây tranh chấp job hoặc in trùng.

### An toàn / giới hạn
- Không nên đưa token production vào máy thử không tin cậy.
- Luôn giữ bản C# làm dự phòng. Chỉ chuyển máy công ty sau khi đối chiếu PDF, trang, dấu 完, giấy in thực, heartbeat và cập nhật.
- `submitted` chỉ có nghĩa đã gửi tới spooler; không chứng minh in xong.
- Nếu có lỗi ở PDF chunk thứ hai, **không tự thử lại** vì chunk đầu có thể đã in.
- Script PowerShell xuất PDF cần Windows PowerShell 5.1 và Excel đã cài. Rust native không thể tự thay thế Excel 2013 COM nếu cần giữ chính xác bố cục XLSX.


## Cập nhật: System Tray, Settings kiểu C#, tự cập nhật Rust

**Tên file:** Mở **Print-Agent-Rust.exe** để có biểu tượng System Tray (có thể nằm sau dấu ^ cạnh đồng hồ). **Print-Agent-Rust-Core.exe** là tiến trình thực thi nhận lệnh và in, không có giao diện riêng. Hai file phải nằm cùng thư mục.

**Print-Settings-Rust.exe**:
- Danh sách máy in Windows, chọn mặc định hoặc tên máy in cụ thể, nút Làm mới.
- Chế độ in: Xuất PDF rồi in (pdfprint) hoặc Chỉ lưu PDF (pdf).
- Nút chọn template XLSX, thư mục PDF, SumatraPDF.exe.
- Chỉnh tên hiển thị trên web, URL, Token, polling, tối đa trang/PDF.
- Checkbox tự động cập nhật và khởi động cùng Windows. Khởi động cùng Windows chỉ mở biểu tượng khay, KHÔNG tự bắt đầu in.

**Auto-update Rust:** GitHub Actions tạo tag rust-v0.4.N, Release chứa fujifilm-rust-update.zip và fujifilm-rust-update.sha256. Print-Updater-Rust.exe kiểm tra bản phát hành mới 6 giờ/lần khi bật cấu hình, không cập nhật nếu Print-Agent-Rust-Core.exe đang chạy và chỉ cài khi SHA-256 đúng. Script cài đặt giữ config.ini, agent-id.txt, XLSX, PDF, log. Cần cài thủ công một lần đầu tiên. Không dùng Release của C#.

**Thử an toàn:** Không chạy Rust Core --enable-live trên máy công ty cùng lúc với C# nếu sử dụng cùng Agent ID. Mở Print-Agent-Rust.exe, nhấp phải icon rồi chọn Settings. Chức năng Bắt đầu nhận lệnh (in thật) yêu cầu xác nhận. Trước đó thử PDF bằng Print-Agent-Rust-Core.exe --test-pdf với PrintMode=pdf.

**Phụ thuộc:** Excel 2013 để xuất PDF, SumatraPDF để in, driver Windows và PowerShell 5.1. EXE chưa ký số có thể hiện cảnh báo SmartScreen.

**Kiểm thử:** Cần xác minh build GitHub Actions Success và kiểm tra thực tế trên Windows. Chưa xác nhận in giấy thật hoặc update xuyên phiên bản.
