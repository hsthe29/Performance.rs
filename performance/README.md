# LLM API Performance Benchmark Tool

Tool benchmark hiệu suất API LLM (OpenAI-compatible `/v1/completions`) được viết bằng Rust.

## Tính năng

- **Time to First Token (TTFT)**: Đo thời gian từ khi gửi request đến khi nhận token đầu tiên
- **Time per Output Token (TPOT)**: Đo thời gian trung bình giữa các tokens
- **Throughput (TPS)**: Tính toán tokens per second
- **Concurrent Requests**: Hỗ trợ test với nhiều requests đồng thời
- **Retry Logic**: Tự động retry khi request thất bại
- **Flexible Output**: Xuất kết quả ra CSV hoặc JSONL

## Cài đặt

```bash
# Clone repository
git clone <repo-url>
cd performance

# Build project
cargo build --release
```

## Cách sử dụng

### 1. Chuẩn bị file config

Tạo file YAML config (ví dụ: `configs/scenario-1.yaml`):

```yaml
base_url: "http://your-api-endpoint/v1"
api_key: "your-api-key"
model: "your-model-name"

output_prefix_name: benchmark-test
output_dir: results
save_format: csv  # hoặc jsonl
stream_writing: true

runs: 10
num_concurrent_requests: 10

cases:
  - input_tokens: 128
    output_tokens: 128
  - input_tokens: 1000
    output_tokens: 1000
  # Thêm các test cases khác...
```

### 2. Chuẩn bị file input text

Đảm bảo có file `input.txt` trong thư mục gốc chứa văn bản để generate prompts.

### 3. Chạy benchmark

```bash
# Chạy với config file
cargo run --release configs/scenario-1.yaml

# Hoặc nếu đã build
./target/release/performance configs/scenario-1.yaml
```

## Workflow

### Phase 1: Generate Prompts
- Chọn ngẫu nhiên 5-10 từ từ `input.txt`
- Gọi API để generate text có số tokens chính xác bằng `input_tokens`
- Lưu prompts đã chuẩn bị cho từng test case

### Phase 2: Run Benchmarks
- Chạy concurrent requests cho mỗi test case
- Đo lường TTFT, TPOT từ streaming response
- Tính toán throughput và các metrics khác

### Phase 3: Write Results
- Xuất kết quả ra file CSV/JSONL
- In summary trên console

## Output Format

### CSV Output
```csv
@timestamp,model,input_tokens,output_tokens,runs,num_concurrent_requests,TTFT (ms),TPOT (ms),Throughput (TPS)
2024-01-01T10:00:00Z,qwen3-235b-stress-test,128,128,10,10,120.5,15.2,658.0
```

### Metrics giải thích
- **TTFT (ms)**: Time to First Token - thời gian từ request đến token đầu tiên
- **TPOT (ms)**: Time per Output Token - thời gian trung bình giữa các tokens
- **Throughput (TPS)**: Tokens per Second = `1000/TPOT * num_concurrent_requests`

## Cấu hình nâng cao

### Retry Logic
- Tool tự động retry tối đa 3 lần khi request thất bại
- Sử dụng exponential backoff (100ms, 200ms, 400ms)

### Concurrent Requests
- Mỗi run sẽ gửi `num_concurrent_requests` requests đồng thời
- Tính toán metrics trung bình từ các requests thành công

### Error Handling
- Log các requests thất bại
- Tiếp tục benchmark với các requests thành công
- Báo cáo số lượng requests failed

## Requirements

- Rust 1.70+
- Network access đến API endpoint
- File `input.txt` chứa văn bản tiếng Việt

## Examples

```bash
# Test với config mẫu
cargo run --release configs/scenario-1.yaml

# Test với endpoint local
# Sửa base_url trong config thành "http://localhost:8000/v1"
cargo run --release configs/local-test.yaml
```

## Troubleshooting

1. **Build errors**: Đảm bảo Rust version >= 1.70
2. **Network errors**: Kiểm tra base_url và api_key
3. **File not found**: Đảm bảo `input.txt` tồn tại
4. **Token counting**: Tool dựa vào response từ API, không validate token count

## Performance Tips

- Sử dụng `--release` build để có hiệu suất tốt nhất
- Điều chỉnh `num_concurrent_requests` dựa trên khả năng của server
- Monitor network và server resources trong quá trình test
