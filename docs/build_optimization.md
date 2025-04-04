## **Link Time Optimizations**

This project is optimized for faster build and link times using high-performance linkers. Faster linking improves compilation speed, especially for large projects.

### **Supported Linkers by Platform**

- **Linux (`x86_64-unknown-linux-gnu`)** – [`mold`](https://github.com/rui314/mold): a fast, parallel linker.  
- **macOS (`x86_64-apple-darwin`)** – [`lld`](https://lld.llvm.org/): the LLVM linker.  
- **Windows (`x86_64-pc-windows-msvc`)** – [`lld-link`](https://lld.llvm.org/): MSVC-compatible linker from LLVM.

---

## **Installation Instructions**

### Linux
To install the `mold` linker, you can use `apt` package manager ship with Debian and Debian-based distributions.
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
