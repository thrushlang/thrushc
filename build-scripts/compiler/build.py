import urllib.request
import platform
import shutil
import socket
import json
import sys
import os

from typing import Dict, List, Any
from functools import lru_cache

if __name__ == "__main__":
    LLVM_C_API_GITHUB_RELEASES: str = "https://api.github.com/repos/thrushlang/toolchains/releases"

    SYSTEM: str = platform.system().lower()

    HOME: str = os.environ["HOME"] if SYSTEM == "linux" else os.environ["APPDATA"].replace("\\", "/")

    FINAL_BUILD_PATH: str = os.path.join(HOME, "thrushlang", "backends", "llvm", "build")

    os.makedirs(FINAL_BUILD_PATH, exist_ok= True)

    @lru_cache(maxsize= 10)
    def has_internet() -> bool:

        try:
            socket.create_connection(("8.8.8.8", 53), timeout= 3)
            return True
        except: return False

    def make_request(url: str) -> Dict[str, Any]:

        with urllib.request.urlopen(url) as get:
            raw_text = get.read().decode('utf-8')
            response_json = json.loads(raw_text)
            
            return response_json
        
    def get_llvm_c_apis(github_api_releases: Dict[str, Any]) -> List[Dict[str, Any]]:

        llvm_c_apis: List[Dict[str, Any]] = [llvm_c_api for llvm_c_api in github_api_releases if isinstance(llvm_c_api, dict) and llvm_c_api.get("tag_name") == "LLVM-C"]

        if len(llvm_c_apis) == 0:

            print(f"LLVM-C APIs tagname no exists in \"{LLVM_C_API_GITHUB_RELEASES}\".")
            sys.exit(1)

        return llvm_c_apis[0]["assets"]
    
    def get_llvm_c_api(llvm_c_apis: List[Dict[str, Any]]) -> tuple[str, str]:

        system_llvm_c_apis: List[tuple[str, str]] = [(llvm_c_api["name"], llvm_c_api["browser_download_url"]) for llvm_c_api in llvm_c_apis if llvm_c_api["name"].find(platform.system().lower()) != -1]

        if len(system_llvm_c_apis) == 0:

            print(f"Not found precompiled LLVM-C APIs for \"{platform.system()}\" operating system.")
            sys.exit(1)

        return get_latest(system_llvm_c_apis)
    
    def get_latest(system_llvm_c_apis: List[tuple[str, str]]) -> tuple[str, str]:

        versions: List[tuple[int, int]] = []

        for index, llvm_c_api in enumerate(system_llvm_c_apis): versions.append((sum([int(v) for v in list(llvm_c_api[0]) if v.isdigit()]), index))

        return system_llvm_c_apis[max(versions)[1]]

    if not has_internet():
        print("You are not connected to the internet.")
        sys.exit(1)

    def build_dependencies():

        if not has_internet():
            print("You are not connected to the internet.")
            sys.exit(1)

        print("Building dependencies for The Thrush Compiler...")
        
        GITHUB_LLVM_C_API: tuple[str, str] = get_llvm_c_api(get_llvm_c_apis(make_request(LLVM_C_API_GITHUB_RELEASES)))
        LLVM_C_API_FILENAME: str = GITHUB_LLVM_C_API[0]
        LLVM_C_API_URL: str = GITHUB_LLVM_C_API[1]
        TEMPORAL_BUILD_PATH: str = os.path.abspath("thrushc-build")
        LLVM_C_COMPRESSED_FILE: str = os.path.join(TEMPORAL_BUILD_PATH, LLVM_C_API_FILENAME)

        os.makedirs(FINAL_BUILD_PATH, exist_ok= True)
        os.makedirs(TEMPORAL_BUILD_PATH, exist_ok= True)

        if not os.path.exists(LLVM_C_COMPRESSED_FILE): 
            urllib.request.urlretrieve(LLVM_C_API_URL, LLVM_C_COMPRESSED_FILE)

        tar: int = os.system(f"tar xvf \"{LLVM_C_COMPRESSED_FILE}\" -C \"{FINAL_BUILD_PATH}\" --strip-components=1") if SYSTEM == "linux" else os.system(f"tar xvf \"{LLVM_C_COMPRESSED_FILE}\" -C \"{FINAL_BUILD_PATH}\"")

        if tar > 0:
            print("Failed to install the LLVM-C API.")
            sys.exit(1)

        match input("\nDo you want to save the precompiled LLVM-C-API? (yes/no): ").lower():
            case "no": shutil.rmtree(TEMPORAL_BUILD_PATH, ignore_errors= True)

        print("\nDependencies are ready to compile. Use 'cargo clean' and 'cargo run' now.")

        sys.exit(0)

    if SYSTEM in ["windows", "linux"]: build_dependencies()

    print(f"Usage: {"py" if SYSTEM == "windows" else "python3"} build.py")
    print(f"Available operating systems: Linux & Windows")

    sys.exit(1)
