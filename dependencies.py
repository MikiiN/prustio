import click

import shutil
import subprocess

def platformio_installed() -> bool:
    return shutil.which("platformio") is not None


def cargo_installed() -> bool:
    return shutil.which("cargo") is not None


def install_cargo():
    url = "https://sh.rustup.rs"
    try:
        subprocess.run(
            ["curl", "--proto", "=https", "--tlsv1.2", "-sSf", url, "|", "sh", "-s", "--", "-y"],
            shell=True,
            check=True,
        )
    except subprocess.CalledProcessError as e:
        raise click.ClickException("Failed to install Cargo")