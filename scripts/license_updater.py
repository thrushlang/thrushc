import os
import re
import sys

YEAR = "2026"
AUTHOR = "Stevens Benavides"
PREFIX = "thrustc_"
REGEX_COPYRIGHT = r"Copyright \(C\) \d{4}"

def update_thrustc_workspaces(base_path: str):

    if not os.path.exists(base_path):

        print(f"'{base_path}' doesn't exist!")
        sys.exit(1)
        
    updated = 0
    
    print(f"Processing '{PREFIX}*' folders in: {base_path}\n")

    for root, dirs, files in os.walk(base_path):
        if 'target' in dirs:
            dirs.remove('target')

        path_parts = os.path.normpath(root).split(os.sep)
        
        if any(part.startswith(PREFIX) for part in path_parts):
            for filename in files:
                if filename.endswith(".rs"):
                    file_path = os.path.join(root, filename)
                    
                    try:
                        with open(file_path, 'r', encoding='utf-8') as f:
                            content = f.read()

                        if "GNU General Public License" in content:
                            new_content = re.sub(REGEX_COPYRIGHT, f"Copyright (C) {YEAR}", content)
                            if new_content != content:
                                with open(file_path, 'w', encoding='utf-8') as f:
                                    f.write(new_content)
                                print(f"Year updated: {file_path}")
                                updated += 1
                            
                    except Exception as e:
                        print(f"Error in {file_path}: {e}")

    print(f"\nFinished. Files updated: {updated}")

if __name__ == "__main__":

    args: list[str] = sys.argv

    if len(args) != 2:

        print("Expected a root path!")
        sys.exit(1)

    update_thrustc_workspaces(args[1])