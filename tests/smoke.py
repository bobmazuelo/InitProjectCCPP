"""End-to-end checks using real toolchains; run after cargo build --locked."""

import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import tomllib
import unittest


ROOT = Path(__file__).resolve().parents[1]
BINARY = Path(os.environ.get("INITPROJECT_BIN", ROOT / "target/debug/InitProject")).resolve()


class GeneratedProjects(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="initproject smoke ")
        self.addCleanup(self.temp.cleanup)
        self.cwd = Path(self.temp.name)
        self.env = os.environ.copy()
        self.env.pop("NOB_PATH", None)

    def command(self, args, cwd=None, success=True):
        result = subprocess.run(
            [str(arg) for arg in args], cwd=cwd or self.cwd, env=self.env,
            text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
            input="", timeout=180,
        )
        if success:
            self.assertEqual(result.returncode, 0, result.stdout)
        else:
            self.assertNotEqual(result.returncode, 0, result.stdout)
        return result.stdout

    def test_invalid_language_and_names_do_not_create_files(self):
        for name, lang in [("unknown", "Java"), ("../escape", "Rust"),
                           ("bad-name", "Python"), ("", "CS"), ("main", "C")]:
            with self.subTest(name=name, lang=lang):
                self.command([BINARY, name, lang, "--no-git"], success=False)
                self.assertEqual(list(self.cwd.iterdir()), [])

    def test_existing_files_are_preserved(self):
        destination = self.cwd / "existing"
        destination.mkdir()
        sentinel = destination / "README.md"
        sentinel.write_text("do not overwrite", encoding="utf-8")
        self.command([BINARY, "existing", "Rust", "--no-git"], success=False)
        self.assertEqual(sentinel.read_text(), "do not overwrite")
        self.assertEqual(list(destination.iterdir()), [sentinel])

    def test_missing_tool_fails_before_creating_destination(self):
        empty_path = self.cwd / "empty_path"
        empty_path.mkdir()
        self.env["PATH"] = str(empty_path)
        self.command([BINARY, "missing", "CS", "--no-git"], success=False)
        self.assertFalse((self.cwd / "missing").exists())

    def test_all_generators_with_and_without_git(self):
        for lang in ["C", "CPP", "CS", "Rust", "PY"]:
            for git in [False, True]:
                with self.subTest(lang=lang, git=git):
                    # Only C/C++ receive a header; other generators must work without it.
                    header = self.cwd / "nob.h"
                    if lang in ["C", "CPP"]:
                        shutil.copyfile(ROOT / "nob.h", header)
                    else:
                        header.unlink(missing_ok=True)
                    name = f"demo_{lang.lower()}_{'git' if git else 'plain'}"
                    args = [BINARY, name, lang]
                    if not git:
                        args.append("--no-git")
                    self.command(args)
                    project = self.cwd / name
                    self.assertEqual((project / ".git").exists(), git)
                    self.assertTrue((project / ".gitignore").is_file())
                    self.assertTrue((project / "README.md").read_text())
                    if git:
                        self.assertEqual(self.command(["git", "branch", "--show-current"], project).strip(), "main")
                    if lang == "C":
                        self.assertIn("42 * 42 == 1764", self.command([project / "bin" / name], project))
                        self.command(["python3", "test.py"], project / "test")
                    elif lang == "CPP":
                        self.assertIn("Hello", self.command([project / "build" / name], project))
                    elif lang == "CS":
                        self.assertTrue((project / f"{name}.csproj").is_file())
                        self.assertIn("Hello", self.command(["dotnet", "run", "--no-build"], project))
                    elif lang == "Rust":
                        self.assertTrue((project / "Cargo.lock").is_file())
                        self.assertIn("Hello", self.command(["cargo", "run", "--locked", "--quiet"], project))
                    else:
                        metadata = tomllib.loads((project / "pyproject.toml").read_text())
                        self.assertEqual(metadata["project"]["requires-python"].replace(" ", ""), ">=3.14,<3.15")
                        self.assertEqual((project / ".python-version").read_text().strip(), "3.14")
                        self.assertTrue((project / "uv.lock").is_file())
                        self.command(["uv", "run", "--frozen", "python", "-c",
                                      "import sys; assert sys.version_info[:2] == (3, 14)"], project)
                        self.assertIn("Hello", self.command(["uv", "run", "--frozen", "python", "main.py"], project))

    def test_relative_header_path_and_empty_destination(self):
        headers = self.cwd / "headers"
        headers.mkdir()
        shutil.copyfile(ROOT / "nob.h", headers / "nob.h")
        self.env["NOB_PATH"] = "headers/nob.h"
        (self.cwd / "empty").mkdir()
        self.command([BINARY, "empty", "C", "--no-git"])
        self.assertEqual((self.cwd / "empty/nob.h").read_bytes(), (headers / "nob.h").read_bytes())

    def test_parent_workspaces_are_unchanged(self):
        manifests = {
            "Cargo.toml": '[workspace]\nmembers = []\n',
            "pyproject.toml": '[project]\nname = "parent"\nversion = "0.1.0"\nrequires-python = ">=3.14"\n\n[tool.uv.workspace]\nmembers = []\n',
        }
        for path, content in manifests.items():
            (self.cwd / path).write_text(content, encoding="utf-8")
        for name, lang in [("childrust", "RS"), ("childpython", "Python3.14")]:
            self.command([BINARY, name, lang, "--no-git"])
        for path, content in manifests.items():
            self.assertEqual((self.cwd / path).read_text(), content)
        self.assertFalse((self.cwd / "Cargo.lock").exists())
        self.assertFalse((self.cwd / "uv.lock").exists())


if __name__ == "__main__":
    unittest.main(verbosity=2)
