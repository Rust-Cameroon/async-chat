# Running the Local CI Pipeline Script

Follow these steps to run the CI pipeline locally using the `ci-local.sh` script:

## Prerequisites
Ensure you have the following installed:
- [Rust](https://www.rust-lang.org/tools/install) (with `cargo` command available)
- [cargo-audit](https://github.com/RustSec/cargo-audit) (for security checks)

## Steps to Run the Script

1. **Clone the Repository** (if not already cloned):
   ```bash
   git clone <repository-url>
   cd async-chat
   ```
2. **Make the Script Executable** (if not already executable):
```bash
chmod +x ci-local.sh
```
3. **Run the CI Script Locally:**
```bash
./ci-local.sh
```
This will execute the following steps:

- Check if formatting issues exist (`cargo fmt --check`)

- Build the project (`cargo build`)

- Run tests (`cargo nextest run`)

- Perform lint checks (`cargo clippy`)

- Check API documentation (`cargo doc`)

- Run a security audit (`cargo audit`)

If any issues are detected (e.g., formatting errors), you can fix them by running:
```bash
cargo fmt --all
```
After fixing the issues, re-run the script to ensure everything passes.