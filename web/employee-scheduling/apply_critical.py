#!/usr/bin/env python3

from pathlib import Path
from string import Template
from typing import List
import multiprocessing
import os.path
import subprocess

SCRIPT_DIR: str = os.path.dirname(os.path.realpath(__file__))
DIST_DIR: str = os.path.join(SCRIPT_DIR, "dist")
COMMAND = Template("critical $source --base $base --inline | sponge $source")


def run_command(command: str):
    subprocess.check_call(command, shell=True)


def main():
    commands: List[str] = []
    for path in Path(DIST_DIR).rglob("*.html"):
        source = path.absolute()
        command: str = COMMAND.substitute(source=source, base=DIST_DIR)
        print("processing %s..." % (source,))
        commands.append(command)
    with multiprocessing.Pool() as pool:
        pool.map(run_command, commands)


if __name__ == "__main__":
    main()
