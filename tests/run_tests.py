import os
import subprocess

def print_success(test_file):
    print(f"\033[32m[SUCCESS] {test_file}\033[0m")

def print_failure(test_file):
    print(f"\033[31m[FAILURE] {test_file}\033[0m")

def run_test(test_path):
    result = subprocess.run([test_path], capture_output=True, text=True)
    return result.stdout.strip()

test_dir = "."
rpack_path = "./../rpack"

for test_file in os.listdir(test_dir):
    if test_file.endswith(".elf"):
        test_path = os.path.join(test_dir, test_file)
        
        packed_file = test_path + ".packed"
        packing_result = subprocess.run([rpack_path, test_path], capture_output=True, text=True)

        if "Success: the file has been successfully packed." not in packing_result.stdout:
            print_failure(f"Packing failed for {test_file}")
            continue
        
        subprocess.run(["chmod", "+x", packed_file])
        
        original_output = run_test(test_path)
        packed_output = run_test(packed_file)
        
        if original_output == packed_output:
            print_success(test_file)
        else:
            print_failure(test_file)
    try:
        os.remove(packed_file)
    except:
        pass
