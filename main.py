import click

from importlib.metadata import version, PackageNotFoundError

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
    pass


if __name__ == '__main__':
    entry_point()