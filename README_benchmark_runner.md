# Benchmark Runner Scripts

Các script Python để chạy benchmark tự động cho Performance.rs tool.

## Scripts có sẵn

### 1. `run_benchmark.py` (Phiên bản đầy đủ)
Script với đầy đủ tính năng:
- ✅ Logging chi tiết vào file
- ✅ Validation môi trường
- ✅ Error handling robust
- ✅ Progress tracking
- ✅ Summary report
- ✅ Timeout handling (30 phút mỗi config)

### 2. `run_benchmark_simple.py` (Phiên bản đơn giản)
Script gọn nhẹ, dễ hiểu:
- ✅ Try-catch handling
- ✅ Progress display
- ✅ Simple error reporting
- ✅ Timeout handling (30 phút mỗi config)

## Thứ tự chạy configs

Cả hai scripts đều chạy theo thứ tự yêu cầu:

1. `configs/internvl3-scenario-1.yaml`
2. `configs/internvl3-scenario-2.yaml`
3. `configs/internvl3-scenario-3.yaml`
4. `configs/internvl3-scenario-4.yaml`
5. `configs/qwen3-235b-scenario-1.yaml`
6. `configs/qwen3-235b-scenario-2.yaml`
7. `configs/qwen3-235b-scenario-3.yaml`
8. `configs/qwen3-235b-scenario-4.yaml`

## Cách sử dụng

### Chạy script đầy đủ:
```bash
python3 run_benchmark.py
```

### Chạy script đơn giản:
```bash
python3 run_benchmark_simple.py
```

### Cấp quyền execute (nếu cần):
```bash
chmod +x run_benchmark.py
chmod +x run_benchmark_simple.py
```

Sau đó có thể chạy trực tiếp:
```bash
./run_benchmark.py
./run_benchmark_simple.py
```

## Tính năng

### Error Handling
- ✅ Try-catch để bỏ qua lần chạy lỗi
- ✅ Timeout protection (30 phút/config)
- ✅ Continue execution khi gặp lỗi
- ✅ Detailed error reporting

### Monitoring
- ✅ Real-time progress display
- ✅ Execution time tracking
- ✅ Success/failure statistics
- ✅ Final summary report

### Logging (run_benchmark.py only)
- ✅ Log file tự động: `benchmark_run_YYYYMMDD_HHMMSS.log`
- ✅ Console output và file logging
- ✅ UTF-8 encoding hỗ trợ tiếng Việt

## Output

### Console output ví dụ:
```
🚀 Bắt đầu chạy benchmark...
Tổng số configs: 8
--------------------------------------------------

[1/8] Đang chạy: configs/internvl3-scenario-1.yaml
✅ THÀNH CÔNG: configs/internvl3-scenario-1.yaml

[2/8] Đang chạy: configs/internvl3-scenario-2.yaml
❌ LỖI: configs/internvl3-scenario-2.yaml (return code: 1)

...

==================================================
KẾT QUẢ CUỐI CÙNG:
Thành công: 6/8
Thất bại: 2/8
Tỷ lệ thành công: 75.0%
==================================================
```

## Yêu cầu hệ thống

- Python 3.6+
- Performance.rs binary đã build tại `performance/target/release/performance`
- Config files tồn tại trong thư mục `configs/`
- Quyền execute cho performance binary

## Troubleshooting

### Lỗi "Permission denied"
```bash
chmod +x performance/target/release/performance
```

### Lỗi "No such file or directory"
Kiểm tra:
- Performance binary đã được build chưa: `cd performance && cargo build --release`
- Config files có tồn tại trong thư mục `configs/` không

### Timeout errors
Scripts đã set timeout 30 phút cho mỗi config. Nếu cần thời gian lâu hơn, có thể edit timeout trong code:
```python
timeout=1800  # Thay đổi giá trị này (đơn vị: giây)
```

## Command thực tế được chạy

Mỗi config sẽ được execute với command:
```bash
performance/target/release/performance {config_file}
```

Ví dụ:
```bash
performance/target/release/performance configs/internvl3-scenario-1.yaml
