#! /usr/bin/env python
# Test Runner that generates Digital Timing Diagrams using wavedrom-rs for all
# `./tests/**/*.json5` files. The resulting SVGs get placed alongside the
# `json5` files. It will also generate a `./tests/result.html` that indexes and
# shows all the tests.
#
# Usage: ./run.py [path/to/skin.json5] 
# 
# The skin is optional

import os
import sys

HTML_TEMPLATE = """
<html>
    <head>
        <title>WaveDrom-RS Tests</title>
    </head>
    <body>
{}
    </body>
</html>
"""
TEST_FILE_EXTENSION = ".json5"

def build_svg(path, out, skin):
    if skin == None:
        rv = os.system(f"../target/debug/wavedrom -i '{path}' -o '{out}'")
    else:
        rv = os.system(f"../target/debug/wavedrom -i '{path}' -o '{out}' -s '{skin}'")

    if rv != 0:
        print(f"Failed the test at {path}")

class TestFile:
    def __init__(self, path: str) -> None:
        no_ext_path = path[:-len(TEST_FILE_EXTENSION)]
        self.path = path
        self.svg = f"{no_ext_path}.svg"
        self.parts = [
            subpart for subpart in no_ext_path.split('/')
        ]

    def id(self):
        return '__'.join(self.parts)

    def display_name(self):
        dir_replaced = '&nbsp;&#187;&nbsp;'.join(self.parts)
        sub_tests_replaced = '&nbsp;&#8250;&nbsp;'.join(dir_replaced.split('_'))
        return sub_tests_replaced

def main():
    test_files = []

    if len(sys.argv) >= 2:
        skin = sys.argv[1]
    else:
        skin = None

    for root, dirs, files in os.walk(r"."):
        for file in files:
            if file.endswith(TEST_FILE_EXTENSION):
                test_files.append(os.path.join(root, file)[len("./"):])

    test_files.sort()
    test_files = list(map(lambda t: TestFile(t), test_files))

    os.system("cargo build --quiet")

    with open('result.html', 'wt') as result:
        result.write("""
        <html>
            <head>
                <title>WaveDrom-RS Tests</title>
            </head>
            <body>
        """)

        result.write(f"""
        <div style="position: fixed; top: 10px;">
            <select onchange="window.location.hash = this.value;"> 
        """)

        for test_file in test_files:
            result.write(f"""
                <option value="{test_file.id()}">{test_file.display_name()}</option> 
            """)

        result.write(f"""
            </select>
        </div>
        """)
        
        result.write(f"""
        <div>
        """)
        for test_file in test_files:
            build_svg(test_file.path, test_file.svg, skin)

            result.write(f"""
                <div style="padding-top: 40px;" id="{test_file.id()}">
                    <a style="text-decoration: none; color: black;" href="#{test_file.id()}">
                        <h2>{test_file.display_name()}</h2>
                    </a>
                    <img src="{test_file.svg}" />
                </div>
            """)
        result.write(f"""
        </div">
        """)

        result.write("""
            </body>
        </html>
        """)

if __name__ == "__main__":
    main()
