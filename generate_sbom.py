import os
import sys
from typing import Optional, Tuple
from pathlib import Path
import subprocess
import tempfile
from typing import List


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
    return path / (output_file + ".json")


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


def merge_sboms(sbom_files: List[Path], output_file: Path):
    if not sbom_files:
        raise ValueError("The list of SBOM files to merge is empty.")

    # If there's only one SBOM file, just copy it to the output file
    if len(sbom_files) == 1:
        output_file.write_text(sbom_files[0].read_text())
        print(f"Only one SBOM provided, copied directly to {output_file}")
        return

    # Use a temporary directory to store intermediate merged files
    with tempfile.TemporaryDirectory() as tmp_dir:
        tmp_dir_path = Path(tmp_dir)

        # Start with the initial list of SBOM files
        current_files = sbom_files

        # Iteratively merge pairs of files until only one file remains
        while len(current_files) > 1:
            next_round_files = []

            # Merge files in pairs
            for i in range(0, len(current_files), 2):
                if i + 1 < len(current_files):
                    # If we have a pair, merge them
                    left_file = current_files[i]
                    right_file = current_files[i + 1]
                    merged_file = tmp_dir_path / f"merged_{i // 2}.json"

                    command = ["sbommerge", "-o", str(merged_file), str(left_file), str(right_file)]
                    subprocess.run(command, check=True)

                    # Add the merged file to the next round
                    next_round_files.append(merged_file)
                else:
                    # If there's an odd file out, move it to the next round as is
                    next_round_files.append(current_files[i])

            # Move to the next round with the merged results
            current_files = next_round_files

        # The last remaining file is the fully merged SBOM
        final_merged_file = current_files[0]
        final_merged_file.rename(output_file)
        print(f"SBOMs merged successfully into {output_file}")


def main():
    print("        START")
    print(" _______________________")

    base_dir = Path(__file__).resolve().parent

    # directory (base_dir / "foo") | recursive (True/False)
    dirs_to_generate_sbom_for = [
        (base_dir / "rust", False),
        # (base_dir / "src", True),
        # (base_dir / "test", True),
    ]

    sbom_files = generate_sboms(dirs_to_generate_sbom_for)

    # base_dir/examo/sub_proj_sbom.json
    # base_dir/sub_proj_sbom.json
    sbom_files = base_dir.rglob("sub_proj_sbom.json")

    rel_sbom_files = list(map(lambda file: file.relative_to(base_dir), sbom_files))

    all_good: bool
    if sbom_files:
        output_name = "proj_sbom"
        print("Merging SBOM files...")
        all_good = merge_sboms(rel_sbom_files, output_name)
        print("Done merging SBOMs.")
    else:
        print("No SBOM files generated. Something went horribly wrong ;)")
        all_good = False

    # remove all sub_proj.sbom files
    for sbom_file in sbom_files:
        try:
            os.unlink(sbom_file)
        except FileNotFoundError:
            pass

    print(" _______________________")
    print("        DONE")
    sys.exit(0 if all_good else -1)


if __name__ == "__main__":
    main()
