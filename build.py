import platform
import sys
import os
import platform
import urllib.request
import shutil

if __name__ == "__main__":
    platform = (platform.platform()).lower()

    HOME: str = os.environ["HOME"] if "linux" in platform else os.environ["APPDATA"].replace("\\", "/")

    os.makedirs(f"{HOME}/thrushlang/backends/llvm/build", exist_ok= True)

    def build_dependencies_for_linux(): 

        print("Building dependencies for The Thrush Compiler in Linux...")

        if not os.path.exists("thrushc-build"): os.mkdir("thrushc-build")

        wget: int = os.system("wget https://github.com/thrushlang/toolchains/releases/download/LLVM-C/thrushc-llvm-linux-x64-v1.0.0.tar.gz -o thrushc-build/thrushc-llvm-linux-x64-v1.0.0.tar.gz")
        tar: int = os.system("tar xvf thrushc-build/thrushc-llvm-linux-x64-v1.0.0.tar.gz")
        decompress: int = os.system(f"rsync -r -a -mkpath {os.path.abspath("llvm/")} {HOME}//thrushlang/backends/llvm/build")

        if sum([wget, tar, decompress]) > 0:
            print("Failed to install LLVM-C API.")
            sys.exit(1)

        shutil.rmtree("thrushc-build/", ignore_errors= True)

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

    if "linux" in platform:
        build_dependencies_for_linux()
    elif "windows" in platform:
        build_dependencies_for_windows()

    print("Usage: python build.py")
    print(f"Available operating systems: linux, windows, not {platform}")

    sys.exit(1)
