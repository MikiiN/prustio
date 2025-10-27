import click

import os
import subprocess 

from platformio.package.manager.platform import PlatformPackageManager
from platformio.platform.exception import UnknownBoard

import templates.main_rust_template as mrt

from pathlib import Path
from typing import IO, Any



def board_validation(ctx, param, value):
    manager = PlatformPackageManager()
    for id in value:
        try:
            manager.board_config(id)
        except UnknownBoard as e:
            raise click.BadParameter(
                f"Unknown board with ID {id}"
            )
    return value


@click.command("init", help="Initialize new project")
@click.option(
    "--dir", "-d",
    default=os.getcwd(),
    type=click.Path(exists=False, file_okay=False, dir_okay=True, writable=True)
)
@click.option(
    "--name", "-n",
    default="project"
)
@click.option(
    "--board", "-b",
    multiple=True, metavar="ID",
    callback=board_validation 
)
@click.option("--sample-code", is_flag=True)
@click.option("--silent", "-s", is_flag=True)
def project_init(
    dir: str,
    name: str,
    board: str,
    sample_code: bool,
    silent: bool
):
    condition_echo(silent, "Initializing project...")
    # TODO cpu config
    project_dir = Path(dir)
    if not project_dir.exists():
        condition_echo("Creating project directory...")
        project_dir.mkdir()
    
    condition_echo(silent, "Initializing cargo project...")
    try:
        subprocess.run(
            ["cargo", "init", "--name", name],
            cwd=project_dir,
            check=True,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL
        )
    except subprocess.CalledProcessError:
        raise click.ClickException("Failed to initialize Cargo project.")
    
    tool_dir = Path(str(project_dir) + "/.prustio")
    if not tool_dir.exists():
        tool_dir.mkdir()

    pio_dir = Path(str(tool_dir) + "/platformio")
    if not pio_dir.exists():
        pio_dir.mkdir()

    condition_echo(silent, "Initializing platformIO project...")
    try:
        subprocess.run(
            ["pio", "init"],
            cwd=pio_dir,
            check=True,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL
        )
    except subprocess.CalledProcessError:
        raise click.ClickException("Failed to initialize platformIO project.")
    
    generate_main_code(sample_code, project_dir)

    generate_proj_config(project_dir)

    generate_cargo_config_file(project_dir)

    generate_cargo_config_folder(project_dir)

    condition_echo(silent, "Done")


def condition_echo(
    silent: bool,
    message: Any | None = None,
    file: IO[Any] | None = None,
    nl: bool = True,
    err: bool = False,
    color: bool | None = None
) -> None:
    if not silent:
        click.echo(message, file, nl, err, color)


def generate_main_code(sample_code: bool, proj_dir: Path):
    if sample_code:
        content = mrt.get_sample_code_content()
    else:
        content = mrt.get_minimal_content()
    src_file = Path(str(proj_dir) + "/src/main.rs")
    with open(src_file, "w") as f:
        f.write(content)


# TODO generate content
def generate_proj_config(proj_dir: Path):
    cfg_file = Path(str(proj_dir) + "/prustio.toml")
    cfg_file.touch()


# TODO generate content
def generate_cargo_config_file(proj_dir: Path):
    cargo_cfg_file = Path(str(proj_dir) + "/Cargo.toml")
    cargo_cfg_file.touch()


# TODO generate content
def generate_cargo_config_folder(proj_dir: Path):
    cargo_cfg_dir = Path(str(proj_dir) + "/.cargo")
    cargo_cfg_file = Path(str(cargo_cfg_dir) + "/config.toml")
    if not cargo_cfg_dir.exists():
        cargo_cfg_dir.mkdir()
    cargo_cfg_file.touch()