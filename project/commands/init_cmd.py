import click

import os
import subprocess 

from platformio.package.manager.platform import PlatformPackageManager
from platformio.platform.exception import UnknownBoard

from pathlib import Path
from typing import IO, Any


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


def board_validation(ctx, param, value):
    manager = PlatformPackageManager()
    for id in value:
        try:
            manager.board_config(id)
        except UnknownBoard as e:
            raise click.BadParameter(
                f"Unknown board with ID {id}"
            )from ctx
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
    project_dir = Path(dir)
    if not project_dir.exists():
        condition_echo("  Creating project directory...")
        project_dir.mkdir()
    
    condition_echo(silent, "  Initializing cargo...")
    result = subprocess.run(
        ["cargo", "init", "--name", name],
        cwd=project_dir,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL
    )
    tool_dir = Path(str(project_dir) + "/.prustio")
    if not tool_dir.exists():
        tool_dir.mkdir()

    pio_dir = Path(str(tool_dir) + "/platformio")
    if not pio_dir.exists():
        pio_dir.mkdir()

    condition_echo(silent, "  Initializing platformIO...")
    result = subprocess.run(
        ["pio", "init"],
        cwd=pio_dir,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL
    )
    condition_echo(silent, "  Done")