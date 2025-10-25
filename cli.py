import click
from importlib.metadata import version, PackageNotFoundError

PACKAGE_NAME = "prustio"

try:
    __version__ = version(PACKAGE_NAME)
except PackageNotFoundError:
    __version__ = "0.0.0+dev"

@click.group()
@click.version_option(__version__, prog_name=PACKAGE_NAME)
def cli():
    pass


if __name__ == '__main__':
    cli()