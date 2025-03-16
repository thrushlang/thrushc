import sys
import os

try:
   import elevate 
except:
   print("Try to install elevate, across `pip install elevate")
   sys.exit(1)

if __name__ == "__main__":

    elevate.elevate()

    def build_dependencies_for_linux(): 

        print("Building dependencies for the Thrush compiler in Linux...")
        print("Downloading LLVM v17.0.6...")

        wget: int = os.system("wget https://github.com/llvm/llvm-project/releases/download/llvmorg-17.0.6/llvm-project-17.0.6.src.tar.xz")
        
        print("Building and installing LLVM v17.0.6...")
        tar: int = os.system("tar xvf llvm-project-17.0.6.src.tar.xz")

        mkdir: int = os.system("mkdir llvm-project-17.0.6.src/llvm/build")
        cmake: int = os.system("cmake llvm-project-17.0.6.src/llvm/CMakeLists.txt -B llvm-project-17.0.6.src/llvm/build -DCMAKE_BUILD_TYPE=MinSizeRel -DCMAKE_INSTALL_PREFIX=/usr/ -DLLVM_BUILD_BENCHMARKS=FALSE -DLLVM_BUILD_DOCS=FALSE -DLLVM_BUILD_EXAMPLES=FALSE")

        cpu_count: int = os.cpu_count() if os.cpu_count() is not None else 2

        make: int = os.system(f"make -j{cpu_count} -C llvm-project-17.0.6.src/llvm/build install")

        if sum([wget, tar, mkdir, cmake, make]) > 0:
            print("Failed to build LLVM v17.0.6.")
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