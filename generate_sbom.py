import os
import subprocess
from pathlib import Path
from typing import Optional, Tuple




def generate_rust_sbom(path: Path) -> Optional[Path]:
    output_file = "sub_proj_sbom"
    result = subprocess.run(
        ["cargo", "cyclonedx", "--override-filename", str(output_file), "--format", "json"],
        cwd=path,
        capture_output=True,
        text=True
    )

    if not result.returncode == 0:
        # Something went wrong
        print(f"Error generating Rust SBOM for {str(path)}: {result.stderr}")
        return None

    print(f"Generated SBOM for {str(path)}")
    return path / output_file


def generate_kotlin_sbom(path: Path) -> Optional[Path]:
    print("not yet impl!")
    return None


# result = subprocess.run(
#     ["./gradlew", "cyclonedxBom"],
#     cwd=path,
#     capture_output=True,
#     text=True,
#     shell=True
# )
# output_file = os.path.join(path, "build/reports/bom.xml")
# if (not result.returncode == 0) or os.path.exists(output_file):
#     # Something went wrong again
#     print(f"Error generating Kotlin SBOM for {path}: {result.stderr}")
#     return None
#
# print(f"Generated SBOM for {path}")
# return output_file


def generate_sboms(path_for_recursive: list[Tuple[Path, bool]]) -> list[Path]:
    sbom_able_projects = []
    print("Finding folders that an SBOM file can be generated for")
    for tgf_dir, recursive in path_for_recursive:
        # User gave file name, why would he do that?
        if not tgf_dir.is_dir():
            print(f"??? '{str(tgf_dir)}' is not a directory!")
            continue

        if recursive:
            for root, dirs, files in os.walk(tgf_dir):
                root_path = Path(root)

                if 'Cargo.toml' in files:
                    sbom_able_projects.append((generate_rust_sbom, root_path))
                elif 'build.gradle' in files:
                    sbom_able_projects.append((generate_kotlin_sbom, root_path))
        else:
            files_of_dir = os.listdir(tgf_dir)

            if 'Cargo.toml' in files_of_dir:
                sbom_able_projects.append((generate_rust_sbom, tgf_dir))
            elif 'build.gradle' in files_of_dir:
                sbom_able_projects.append((generate_kotlin_sbom, tgf_dir))
            else:
                print("??? No 'Cargo.toml' or 'build.gradle' in none-recursive-search dir")

    if not sbom_able_projects:
        print("!!!   No projects found")
        return []

    print("Generating SBOMs for:")
    for _, path in sbom_able_projects:
        print(" -{}".format(str(path)))

    l = list(filter(lambda x: x, map(lambda x: x[0](x[1]), sbom_able_projects)))

    return l


def merge_sboms(sbom_files: list[Path], output_file: Path, rename_old_sbom=True):
    # Merge all SBOM files to output file
    pass


def main():
    print("        START")
    print(" _______________________")

    base_dir = Path(__file__).resolve().parent

    # directory (base_dir / "foo") | recursive (True/False)
    dirs_to_generate_sbom_for = [
        (base_dir / "rust", True),
        # (base_dir / "src", True),
        # (base_dir / "test", True),
    ]

    sbom_files = generate_sboms(dirs_to_generate_sbom_for)
    if sbom_files:
        output_name = base_dir / "proj_sbom"
        print("Merging SBOM files...")
        merge_sboms(sbom_files, output_name)
        print("Done merging SBOMs.")
    else:
        print("No SBOM files generated. Something went horribly wrong ;)")

    # remove all sub_proj.sbom files
    for sbom_file in sbom_files:
        try:
            os.remove(sbom_file)
        except FileNotFoundError:
            pass

    print(" _______________________")
    print("        DONE")


if __name__ == "__main__":
    main()
