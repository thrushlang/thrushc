import platform
import sys
import os
import urllib.request
import shutil

if __name__ == "__main__":
    SYSTEM: str = platform.system().lower()

    HOME: str = os.environ["HOME"] if SYSTEM == "linux" else os.environ["APPDATA"].replace("\\", "/")

    FINAL_BUILD_PATH: str = os.path.join(HOME, "thrushlang", "backends", "llvm", "build")

    os.makedirs(FINAL_BUILD_PATH, exist_ok= True)

    def build_dependencies_for_linux():
        COMPRESS_FILE: str = "thrushc-llvm-linux-x64-v1.0.0.tar.gz"
        GITHUB_BUILD_FILE: str = f"https://github.com/thrushlang/toolchains/releases/download/LLVM-C/{COMPRESS_FILE}"
        TEMP_BUILD_DIR: str = "thrushc-build"
        TEMP_BUILD_PATH: str = os.path.join(TEMP_BUILD_DIR, "thrushc-llvm-linux-x64-v1.0.0.tar.gz")

        print("Building dependencies for The Thrush Compiler in Linux...")

        os.makedirs(TEMP_BUILD_DIR, exist_ok=True)
        os.makedirs(os.path.abspath('llvm/'), exist_ok=True)

        wget: int = os.system(f"wget {GITHUB_BUILD_FILE} -O {TEMP_BUILD_PATH} -o /dev/null")
        tar: int = os.system(f"tar xvf {TEMP_BUILD_PATH} > /dev/null")
        decompress: int = os.system(f"rsync -r -a -mkpath {os.path.abspath('llvm/')} {FINAL_BUILD_PATH}")

        for action in [wget, tar, decompress]:
            if action != 0:
                print(f"Error in {action}")
                sys.exit(1)

        shutil.rmtree(TEMP_BUILD_DIR, ignore_errors= True)

        print("Dependencies are ready to compile. Use 'cargo clean' and 'cargo run' now.")

        sys.exit(0)

    def build_dependencies_for_windows():

        print("Building dependencies for The Thrush Compiler in Windows...")

        if not os.path.exists("thrushc-build"): os.mkdir("thrushc-build")

        urllib.request.urlretrieve("https://github.com/thrushlang/toolchains/releases/download/LLVM-C/thrushc-llvm-windows-x64-v1.0.0.tar.xz", "thrushc-build/thrushc-llvm-windows-x64-v1.0.0.tar.xz")

        tar: int = os.system(f"tar -xJf thrushc-build/thrushc-llvm-windows-x64-v1.0.0.tar.xz -C {HOME}/thrushlang/backends/llvm/build/")

        if tar > 0:
            print("Failed to install the LLVM-C API.")
            sys.exit(1)

        shutil.rmtree("thrushc-build/", ignore_errors= True)

        print("Dependencies are ready to compile. Use 'cargo clean' and 'cargo run' now.")

        sys.exit(0)

    match SYSTEM:
        case "linux":
            build_dependencies_for_linux()
        case "windows":
            build_dependencies_for_windows()

    print(f"Usage: {"py" if SYSTEM == "windows" else "python3"}")
    print(f"Available operating systems: linux, windows, not {SYSTEM}")

    sys.exit(1)
