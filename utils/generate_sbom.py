import os
from typing import Tuple
from pathlib import Path
import subprocess

sub_bom_file_name = "bom"
output_file_name = "sbom"

def generate_rust_sbom(path: Path):
    result = subprocess.run(
        ["cargo", "cyclonedx", "--override-filename", str(sub_bom_file_name), "--format", "json", "--top-level"],
        cwd=path,
        capture_output=True,
        text=True
    )

    if not result.returncode == 0:
        # Something went wrong
        print(f"Error generating Rust SBOM for {str(path)}: {result.stderr}")
        return

    print(f"Generated SBOM for {str(path)}")
    return


def generate_kotlin_sbom(path: Path):
    result = subprocess.run(
        ["./gradlew", "cyclonedxBom"],
        cwd=path,
        capture_output=True,
        text=True
    )
    if (not result.returncode == 0):
        # Something went wrong again
        print(f"Error generating Kotlin SBOM for {path}: {result.stderr}")
        return

    print(f"Generated SBOM for {path}")
    return


def generate_nix_sbom():
    result = subprocess.run(
        ["nix", "build", ".#toolsSbom", "-o", sub_bom_file_name + ".json"],
        capture_output=True,
        text=True
    )

    if (not result.returncode == 0):
        # Something went wrong
        print(f"Error generating SBOM for nix: {result.stderr}")
        return None

    print(f"Generated SBOM for nix")

    return


def generate_sboms(path_for_recursive: list[Tuple[Path, bool]]) -> list[Path]:
    generate_nix_sbom()

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
                elif 'build.gradle.kts' in files:
                    sbom_able_projects.append((generate_kotlin_sbom, root_path))
        else:
            files_of_dir = os.listdir(tgf_dir)

            if 'Cargo.toml' in files_of_dir:
                sbom_able_projects.append((generate_rust_sbom, tgf_dir))
            elif 'build.gradle.kts' in files_of_dir:
                sbom_able_projects.append((generate_kotlin_sbom, tgf_dir))
            else:
                print("??? No 'Cargo.toml' or 'build.gradle.kts' in none-recursive-search dir")

    if not sbom_able_projects:
        print("!!!   No projects found")
        return []

    print("Generating SBOMs for:")
    for _, path in sbom_able_projects:
        print(" -{}".format(str(path)))

    return list(filter(lambda x: x, map(lambda x: x[0](x[1]), sbom_able_projects)))

def merge_sboms(rel_sbom_files):
    # Example usage:
    # cyclonedx merge --input-files ./result.json ./rust/example-ebpf/example-ebpf.cdx.json --output-format json
    result = subprocess.run(
        ["cyclonedx", "merge", "--input-format", "json", "--output-format", "json", "--output-file", output_file_name + ".json", "--input-files"] + rel_sbom_files,
        capture_output=True,
        text=True
    )

    if not result.returncode == 0:
        # Something went wrong
        print(f"Error merging SBOMs: {result.stderr}")
        return None

def main():
    print("        START")
    print(" _______________________")

    # traverse back to base project folder
    base_dir = Path(__file__).resolve().parent.parent

    # usage:
    # directory (base_dir / "foo") | recursive (True/False)
    dirs_to_generate_sbom_for = [
        (base_dir / "rust", False),
        (base_dir / "frontend", False),
    ]

    generate_sboms(dirs_to_generate_sbom_for)

    # example paths:
    # base_dir/exampl/bom.json
    # base_dir/bom.json
    sbom_files = base_dir.rglob(sub_bom_file_name + ".json")

    rel_sbom_files = list(map(lambda file: file.relative_to(base_dir), sbom_files))

    merge_sboms(rel_sbom_files)

    # remove all bom.json files
    for sbom_file in rel_sbom_files:
        try:
            os.unlink(sbom_file)
        except FileNotFoundError:
            pass

    print(" _______________________")
    print("        DONE")


if __name__ == "__main__":
    main()
