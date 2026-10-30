"""Check that every ai-rulez installation takes the version this checkout pins or expects."""

from pathlib import Path
import re
import tomllib
import unittest


def _locked_ai_rulez_installs(path: Path, *, require_python: bool) -> int:
    source = path.read_text()
    for forbidden in ("xberg-io/actions/setup-ai-rulez@", "ai-rulez-version:", "source-revision:"):
        assert forbidden not in source, f"{path} bypasses the locked mise installer: {forbidden}"
    assert "github:Goldziher/ai-rulez@" not in source, f"{path} overrides the mise.toml ai-rulez pin"
    assert re.search(r"MISE_\w*AI_RULEZ\w*_VERSION", source) is None, f"{path} overrides the mise.toml pin"
    assert "AI_RULEZ_VERSION" not in source, f"{path} overrides the mise.toml ai-rulez pin"
    installs = 0
    for match in re.finditer(r"(?m)^ +install-args: ([^\n]+)", source):
        tools = match.group(1).split()
        if "github:Goldziher/ai-rulez" not in tools:
            continue
        if require_python:
            assert "python" in tools, f"{path} omits Python from the CI governance installation"
        actions = re.findall(r"(?m)^ +(?:- )?uses: ([^\n]+)", source[: match.start()])
        assert actions and actions[-1] == "xberg-io/actions/setup-mise@v1", f"{path} uses a non-mise installer"
        installs += 1
    return installs


class GovernanceRuntimeTests(unittest.TestCase):
    """mise.toml pins ai-rulez exactly, so the tasks run it without a runtime version check."""

    def test_should_use_locked_mise_for_ci_and_release_installations(self) -> None:
        """Three CI installs and six release installs use the pinned mise toolchain."""
        root = Path(__file__).resolve().parents[2]
        mise = (root / "mise.toml").read_text()
        ai_rulez_pin = tomllib.loads(mise)["tools"]["github:Goldziher/ai-rulez"]
        assert re.fullmatch(r"\d+\.\d+\.\d+", ai_rulez_pin) is not None
        task_sources = [mise, *(path.read_text() for path in (root / "mise-tasks").rglob("*") if path.is_file())]
        assert len(task_sources) > 1, "mise-tasks/ holds no task files"
        for source in task_sources:
            assert "AI_RULEZ_VERSION" not in source, "a task overrides the mise.toml ai-rulez pin"
        release_paths = {"publish.yaml", "plugin-artifacts.yaml"}
        installs = {"ci": 0, "release": 0}
        for path in (root / ".github/workflows").glob("*"):
            kind = "release" if path.name in release_paths else "ci"
            installs[kind] += _locked_ai_rulez_installs(path, require_python=kind == "ci")
        assert installs == {"ci": 3, "release": 6}, installs


if __name__ == "__main__":
    unittest.main()
