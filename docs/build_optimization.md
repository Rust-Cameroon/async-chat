# Link Time Optimization (LTO) Setup  

This project is configured for faster build and link times using high-performance linkers. These linkers significantly reduce compilation time, improving the development workflow, especially for large Rust projects. 

### **Supported Linkers by Platform**

- **Linux (`x86_64-unknown-linux-gnu`)**: Uses [**mold**](https://github.com/rui314/mold), a lightning-fast linker optimized for multicore performance. 
- **macOS (`x86_64-apple-darwin`)**: Uses [**lld**](https://lld.llvm.org/), the LLVM project’s high-performance linker known for it's efficiency.  
- **Windows (`x86_64-pc-windows-msvc`)**: Uses [**lld-link**](https://lld.llvm.org/), the MSVC-compatible linker from the LLVM project known for its performance and compatibility.

---

## **Installation Instructions**

### Linux
To install the `mold` linker, You can use the `apt` package manager available on Debian and Ubuntu-based distributions.
Additionally, clang is required as the linker driver to use `mold`

```sh
sudo apt update
sudo apt install mold clang
```
For other Linux distributions or package managers, refer to the [official mold installation guide](https://github.com/rui314/mold/blob/master/README.md).

### macOS
Ensure Xcode Command Line Tools are installed, then install `lld` via Homebrew:

```sh
xcode-select --install  # Install Xcode Command Line Tools  
brew install lld        # Install lld  
```


### Windows

1. [Download LLVM (Pre-built binaries)](https://github.com/llvm/llvm-project/releases) and install it (choose the version with `lld-link.exe`).
2. Add LLVM to your system PATH:

```powershell
$llvmPath = "C:\Program Files\LLVM\bin"
[Environment]::SetEnvironmentVariable("Path", $Env:Path + ";$llvmPath", [EnvironmentVariableTarget]::Machine)
```

Then restart your terminal.

---

## **Verify Installation**

```sh
mold --version        # Linux
lld --version         # macOS
lld-link --version    # Windows
```

---

## **Test Linker Usage**

For Linux (mold), confirm it’s being used via:

```sh
readelf -p .comment <path-to-executable>
```

Look for:

```
String dump of section '.comment':
  [002b]  mold xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
```
