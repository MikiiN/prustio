import click

from importlib.metadata import version, PackageNotFoundError

import dependencies as dep

from project.cli import cli as cli_project


PACKAGE_NAME = "prustio"

try:
    __version__ = version(PACKAGE_NAME)
except PackageNotFoundError:
    __version__ = "0.0.0+dev"


@click.group(
    commands=[
        cli_project,
    ]
)
@click.version_option(__version__, prog_name=PACKAGE_NAME)
def entry_point():
    if not dep.platformio_installed(): # Just to be sure, should be installed during tool installation
        raise click.ClickException("Missing required dependency: platformIO")
    
    if not dep.cargo_installed():
        click.echo("Can not find cargo tool.")
        if click.confirm("Do you want to install cargo?"):
            click.echo("Installing cargo..")
            dep.install_cargo()
            click.echo("Cargo has been installed successfully")
        else:
            raise click.ClickException("Missing required dependency: cargo")


if __name__ == '__main__':
    entry_point()