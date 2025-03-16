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

    def build_dependencies_for_linux(): 

        print("Building dependencies for the Thrush compiler in Linux...")
        print("Installing LLVM-C API v17.0.6...")

        wget: int = os.system("wget https://github.com/thrushlang/toolchains/releases/download/LLVM-C/thrushc-llvm-linux-x64_86-v1.0.0.tar.gz")
        
        print("Unpacking the precompiled LLVM-C API v17.0.6...")
        tar: int = os.system("tar xvf thrushc-llvm-linux-x64_86-v1.0.0.tar.gz")

        os.system(f"rsync -r -a -mkpath {os.path.abspath("llvm/")} /usr/")

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
