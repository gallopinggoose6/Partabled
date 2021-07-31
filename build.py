#!/usr/bin/env python3

'''Script used to build, run, and test the code. 
Adapted from the build script from https://github.com/rust-osdev/uefi-rs'''

import argparse
import filecmp
import json
import os
from pathlib import Path
import re
import shutil
import subprocess as sp
import sys

## Configurable settings
# Path to workspace directory (which contains the top-level `Cargo.toml`)
WORKSPACE_DIR = Path(__file__).resolve()

# Try changing these with command line flags, where possible
SETTINGS = {
    # Architecture to build for
    'arch': 'x86_64',
    # Configuration to build.
    'config': 'debug',
    # QEMU executable to use
    # Indexed by the `arch` setting
    'qemu_binary': 'qemu-system-x86_64',
    # Path to directory containing `OVMF_{CODE/VARS}.fd` (for x86_64),
    # or `*-pflash.raw` (for AArch64).
    # `find_ovmf` function will try to find one if this isn't specified.
    'ovmf_dir': Path('/usr/share/edk2-ovmf'),
}

# Path to target directory. If None, it will be initialized with information
# from cargo metadata at the first time target_dir function is invoked.
TARGET_DIR = None

def target_dir():
    'Returns the target directory'
    global TARGET_DIR
    if TARGET_DIR is None:
        cmd = ['cargo', 'metadata', '--format-version=1']
        result = sp.run(cmd, stdout=sp.PIPE, check=True)
        TARGET_DIR = Path(json.loads(result.stdout)['target_directory'])
    return TARGET_DIR

def get_target_triple():
    return f'x86_64-unknown-uefi'

def build_dir():
    'Returns the directory where Cargo places the build artifacts'
    return target_dir() / get_target_triple() / SETTINGS['config']

def esp_dir():
    'Returns the directory where we will build the emulated UEFI system partition'
    return build_dir() / 'esp'

def run_tool(tool, *flags):
    'Runs cargo-<tool> with certain arguments.'

    target = get_target_triple()
    cmd = ['cargo', tool, '--target', target, *flags]
    sp.run(cmd, check=True)

def run_build(*flags):
    'Runs cargo-build with certain arguments.'
    run_tool('build', *flags)

def run_clippy(*flags):
    'Runs cargo-clippy with certain arguments.'
    run_tool('clippy', *flags)

def build(*test_flags):
    'Builds the test crate.'

    build_args = [
        *test_flags,
    ]

    if SETTINGS['config'] == 'release':
        build_args.append('--release')

    run_build(*build_args)

    # Copy the built test runner file to the right directory for running tests.
    built_file = build_dir() / 'partabled.efi'

    boot_dir = esp_dir() / 'EFI' / 'Boot'
    boot_dir.mkdir(parents=True, exist_ok=True)

    output_file = boot_dir / 'BootX64.efi'
    shutil.copy2(built_file, output_file)

def clippy():
    'Runs Clippy on all projects'

    run_clippy('--all')

def ovmf_files(ovmf_dir):
    'Returns the tuple of paths to the OVMF code and vars firmware files, given the directory'
    return ovmf_dir / 'OVMF_CODE.fd', ovmf_dir / 'OVMF_VARS.fd'
    
def check_ovmf_dir(ovmf_dir):
    'Check whether the given directory contains necessary OVMF files'
    ovmf_code, ovmf_vars = ovmf_files(ovmf_dir)
    return ovmf_code.is_file() and ovmf_vars.is_file()

def find_ovmf():
    'Find path to OVMF files'

    # If the path is specified in the settings, use it.
    if SETTINGS['ovmf_dir'] is not None:
        ovmf_dir = SETTINGS['ovmf_dir']
        if check_ovmf_dir(ovmf_dir):
            return ovmf_dir
        raise FileNotFoundError(f'OVMF files not found in `{ovmf_dir}`')

    # Check whether the test runner directory contains the files.
    if check_ovmf_dir(WORKSPACE_DIR):
        return WORKSPACE_DIR

    if sys.platform.startswith('linux'):
        possible_paths = [
            # Most distros, including CentOS, Fedora, Debian, and Ubuntu.
            Path('/usr/share/OVMF'),
            # Arch Linux
            Path('/usr/share/ovmf/x64'),
        ]
        for path in possible_paths:
            if check_ovmf_dir(path):
                return path

    raise FileNotFoundError(f'OVMF files not found anywhere')

def run_qemu():
    'Runs the code in QEMU.'

    # Rebuild all the changes.
    build()

    ovmf_code, ovmf_vars = ovmf_files(find_ovmf())

    qemu_monitor_pipe = 'qemu-monitor'

    arch = SETTINGS['arch']

    qemu_flags = [
        # Disable default devices.
        # QEMU by defaults enables a ton of devices which slow down boot.
        '-nodefaults',
    ]

    ovmf_vars_readonly = 'on'
    qemu_flags.extend([
        # Use a modern machine,.
        '-machine', 'q35',

        # Multi-processor services protocol test needs exactly 4 CPUs.
        '-smp', '4',

        # Allocate some memory.
        '-m', '256M',
    ])
    qemu_flags.append('--enable-kvm')
    qemu_flags.extend([
        # Set up OVMF.
        '-drive', f'if=pflash,format=raw,file={ovmf_code},readonly=on',
        '-drive', f'if=pflash,format=raw,file={ovmf_vars},readonly={ovmf_vars_readonly}',

        # Mount a local directory as a FAT partition.
        '-drive', f'format=raw,file=fat:rw:{esp_dir()}',

        # Connect the serial port to the host. OVMF is kind enough to connect
        # the UEFI stdout and stdin to that port too.
        '-serial', 'stdio',

        # Map the QEMU monitor to a pair of named pipes
        '-qmp', f'pipe:{qemu_monitor_pipe}',
    ])

    # Enable debug features
    qemu_flags.extend([
        # Map the QEMU exit signal to port f4
        '-device', 'isa-debug-exit,iobase=0xf4,iosize=0x04',

        # OVMF debug builds can output information to a serial `debugcon`.
        # Only enable when debugging UEFI boot:
        #'-debugcon', 'file:debug.log', '-global', 'isa-debugcon.iobase=0x402',
    ])

    # When running in headless mode we don't have video, but we can still have
    # QEMU emulate a display and take screenshots from it.
    qemu_flags.extend(['-vga', 'std'])

    qemu_binary = SETTINGS['qemu_binary']
    cmd = [qemu_binary] + qemu_flags

    # This regex can be used to detect and strip ANSI escape codes when
    # analyzing the output of the test runner.
    ansi_escape = re.compile(r'(\x9B|\x1B\[)[0-?]*[ -/]*[@-~]')

    # Setup named pipes as a communication channel with QEMU's monitor
    monitor_input_path = f'{qemu_monitor_pipe}.in'
    os.mkfifo(monitor_input_path)
    monitor_output_path = f'{qemu_monitor_pipe}.out'
    os.mkfifo(monitor_output_path)

    # Start QEMU
    qemu = sp.Popen(cmd, stdin=sp.PIPE, stdout=sp.PIPE, universal_newlines=True)
    try:
        # Connect to the QEMU monitor
        with open(monitor_input_path, mode='w') as monitor_input,                  \
             open(monitor_output_path, mode='r') as monitor_output:
            # Execute the QEMU monitor handshake, doing basic sanity checks
            assert monitor_output.readline().startswith('{"QMP":')
            print('{"execute": "qmp_capabilities"}', file=monitor_input, flush=True)
            assert monitor_output.readline() == '{"return": {}}\n'

            # Iterate over stdout...
            for line in qemu.stdout:
                # Strip ending and trailing whitespace + ANSI escape codes
                # (This simplifies log analysis and keeps the terminal clean)
                stripped = ansi_escape.sub('', line.strip())

                # Skip lines which contain nothing else
                if not stripped:
                    continue

                # Print out the processed QEMU output for logging & inspection
                print(stripped)

                # If the app requests a screenshot, take it
                if stripped.startswith("SCREENSHOT: "):
                    reference_name = stripped[12:]

                    # Ask QEMU to take a screenshot
                    monitor_command = '{"execute": "screendump", "arguments": {"filename": "screenshot.ppm"}}'
                    print(monitor_command, file=monitor_input, flush=True)

                    # Wait for QEMU's acknowledgement, ignoring events
                    reply = json.loads(monitor_output.readline())
                    while "event" in reply:
                        reply = json.loads(monitor_output.readline())
                    assert reply == {"return": {}}

                    # Tell the VM that the screenshot was taken
                    print('OK', file=qemu.stdin, flush=True)

                    # Compare screenshot to the reference file specified by the user
                    # TODO: Add an operating mode where the reference is created if it doesn't exist
                    reference_file = WORKSPACE_DIR / 'screenshots' / (reference_name + '.ppm')
                    assert filecmp.cmp('screenshot.ppm', reference_file)

                    # Delete the screenshot once done
                    os.remove('screenshot.ppm')
    finally:
        try:
            # Wait for QEMU to finish
            status = qemu.wait()
        except sp.TimeoutExpired:
            print('Tests are taking too long to run, killing QEMU', file=sys.stderr)
            qemu.kill()
            status = -1

        # Delete the monitor pipes
        os.remove(monitor_input_path)
        os.remove(monitor_output_path)

        # Throw an exception if QEMU failed
        if status != 0 and status != 3:
            raise sp.CalledProcessError(cmd=cmd, returncode=status)

def main():
    'Runs the user-requested actions.'

    # Clear any Rust flags which might affect the build.
    os.environ['RUSTFLAGS'] = ''

    desc = 'Build script for UEFI programs'

    parser = argparse.ArgumentParser(description=desc)

    parser.add_argument('verb', help='command to run', type=str,
                        choices=['build', 'run', 'clippy'])

    parser.add_argument('--target', help='target to build for (default: %(default)s)', type=str,
                        choices=['x86_64', 'aarch64'], default='x86_64')

    parser.add_argument('--verbose', '-v', help='print commands before executing them',
                        action='store_true')

    parser.add_argument('--headless', help='run QEMU without a GUI',
                        action='store_true')

    parser.add_argument('--release', help='build in release mode',
                        action='store_true')

    parser.add_argument('--ci', help='disables some tests which currently break CI',
                        action='store_true')

    opts = parser.parse_args()

    SETTINGS['arch'] = opts.target
    # Check if we need to enable verbose mode
    SETTINGS['verbose'] = opts.verbose
    SETTINGS['headless'] = opts.headless
    SETTINGS['config'] = 'release' if opts.release else 'debug'

    verb = opts.verb

    if verb == 'build':
        build()
    elif verb == 'clippy':
        clippy()
    elif verb == 'run' or verb is None or opts.verb == '':
        # Run the program, by default.
        run_qemu()
    else:
        raise ValueError(f'Unknown verb {opts.verb}')

if __name__ == '__main__':
    try:
        main()
    except sp.CalledProcessError as cpe:
        print(f'Subprocess {cpe.cmd[0]} exited with error code {cpe.returncode}')
        sys.exit(1)
