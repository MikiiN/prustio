import click

from project.commands.init_cmd import project_init


@click.group(
    "project",
    commands=[
        project_init,
    ],
    short_help="Project Manager",
)
def cli():
    pass