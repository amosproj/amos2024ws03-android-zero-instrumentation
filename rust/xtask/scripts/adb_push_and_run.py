#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2024 Felix Hilgers <felix.hilgers@fau.de>
#
# SPDX-License-Identifier: MIT

import subprocess
import sys
import os
import argparse
import signal

def adb_push_and_run(binary_path, extra_args):
    if not os.path.isfile(binary_path):
        print(f"Error: {binary_path} does not exist.")
        sys.exit(1)

    binary_name = os.path.basename(binary_path)

    remote_dir = "/data/local/tmp"
    remote_path = f"{remote_dir}/{binary_name}"
    extra_args = " ".join(extra_args)

    print(f"Pushing {binary_name} to {remote_path} on the Android device...")
    push_command = ["adb", "push", binary_path, remote_path]
    subprocess.run(push_command, check=True)

    print(f"Running {binary_name} on the Android device as root...")
    run_command = ["adb", "shell", "sh", "-c", f"'cd {remote_dir} && su root ./{binary_name} {extra_args}'"]

    try:
        subprocess.run(run_command, check=True)
    except KeyboardInterrupt:
        subprocess.run(["adb", "shell", "su", "root", "pkill", binary_name], check=True)

    print("Execution complete.")

if __name__ == "__main__":
    # Get the binary path and extra arguments
    binary_path = sys.argv[-1]
    extra_args = sys.argv[1:-1]

    # Run the adb push and run process
    adb_push_and_run(binary_path, extra_args)

