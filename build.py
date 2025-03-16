import sys
import os

from typing import List

try:
   import elevate 
except:
   print("Try to install elevate, across `pip install elevate`.")
   sys.exit(1)

if __name__ == "__main__":

    elevate.elevate()

    def get_files(path: str, generated_target: List[str], visited: set = None) -> List[str]:
        if not os.path.exists(path):
            return generated_target

        if visited is None:
            visited = set()

        if path in visited:
            return generated_target

        visited.add(path)

        for name in os.listdir(path):
            full_path = os.path.join(path, name)
            if os.path.isfile(full_path):
                generated_target.append(os.path.abspath(full_path))
            elif os.path.isdir(full_path):
                get_files(full_path, generated_target, visited)

        return generated_target

    def install_linux_llvm_for_thrushc(files: List[List[str]]): 
        
        for index, source in enumerate(files):
            for file in source:
                os.system(f"cp {file} /usr/include/") if index == 0 else os.system(f"sudo cp {file} /usr/lib/") if index == 1 else os.system(f"sudo cp {file} /usr/bin/")

    def build_dependencies_for_linux(): 

        print("Building dependencies for the Thrush compiler in Linux...")
        print("Installing LLVM-C API v17.0.6...")

        wget: int = os.system("wget https://github.com/thrushlang/toolchains/releases/download/LLVM-C/thrushc-llvm-linux-x64_86-v1.0.0.tar.gz")
        
        print("Unpacking the precompiled LLVM-C API v17.0.6...")
        tar: int = os.system("tar xvf thrushc-llvm-linux-x64_86-v1.0.0.tar.gz")

        llvm_c_includes: List[str] = get_files("llvm/include")
        llvm_c_libraries: List[str] = get_files("llvm/lib")
        llvm_c_binaries: List[str] = get_files("llvm/bin")

        install_linux_llvm_for_thrushc([llvm_c_includes, llvm_c_libraries, llvm_c_binaries])

        if sum([wget, tar]) > 0:
            print("Failed to install LLVM-C API v17.0.6.")
            sys.exit(1)

        print("Dependencies are ready to compile. Use cargo clean and cargo run now.")
        
        sys.exit(0)
    
    if len(sys.argv) < 2:
        print("Usage: python build.py <operating-system>")
        print("Available operating systems: linux")
        sys.exit(1)

    if sys.argv[1].lower() == "linux":
        build_dependencies_for_linux()
    
    print("Usage: python build.py <target-operating-system>")
    print("Available operating systems: linux")

    sys.exit(1)
