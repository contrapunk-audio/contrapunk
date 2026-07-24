#!/usr/bin/env python3
import os
import pathlib
import subprocess
import tempfile
import xml.etree.ElementTree as ET

ROOT = pathlib.Path(__file__).resolve().parents[1]


def test_distribution() -> None:
    source = (ROOT / "packaging/macos/Distribution.xml.in").read_text().replace("@VERSION@", "1.2.3")
    root = ET.fromstring(source)
    choices = {choice.attrib["id"]: choice for choice in root.findall("choice")}
    assert choices["contrapunk"].attrib["enabled"] == "false"
    assert choices["contrapunk"].attrib["selected"] == "true"
    assert choices["elixir"].attrib["selected"] == "true"
    assert {ref.attrib["id"] for ref in root.findall("pkg-ref")} == {
        "com.contrapunk.plugins.core",
        "com.contrapunk.plugins.elixir",
    }


def test_macos_removal() -> None:
    paths = {
        "contrapunk": [
            "Library/Audio/Plug-Ins/VST3/Contrapunk.vst3",
            "Library/Audio/Plug-Ins/CLAP/Contrapunk.clap",
            "Library/Audio/Plug-Ins/Components/Contrapunk.component",
            "Library/Audio/Plug-Ins/Components/Contrapunk Guitar.component",
        ],
        "elixir": [
            "Library/Audio/Plug-Ins/VST3/Elixir.vst3",
            "Library/Audio/Plug-Ins/CLAP/Elixir.clap",
        ],
    }
    with tempfile.TemporaryDirectory() as tmp:
        root = pathlib.Path(tmp)
        for product_paths in paths.values():
            for relative in product_paths:
                target = root / relative
                target.mkdir(parents=True)
                (target / "payload").write_text("test")
        env = {**os.environ, "PLUGIN_ROOT": tmp}
        subprocess.run([ROOT / "packaging/macos/uninstall.sh", "--contrapunk"], env=env, check=True)
        assert all(not (root / path).exists() for path in paths["contrapunk"])
        assert all((root / path).exists() for path in paths["elixir"])
        subprocess.run([ROOT / "packaging/macos/uninstall.sh", "--elixir"], env=env, check=True)
        assert all(not (root / path).exists() for path in paths["elixir"])


def test_windows_components_and_removal() -> None:
    source = (ROOT / "packaging/windows/ContrapunkPlugins.nsi").read_text()
    assert "!insertmacro MUI_PAGE_COMPONENTS" in source
    required = source[source.index('SectionGroup /e "Contrapunk'):source.index("SectionGroupEnd")]
    assert required.count("SectionIn RO") == 2
    optional_start = source.index('SectionGroup /e "Elixir')
    optional = source[optional_start:source.index("SectionGroupEnd", optional_start)]
    assert "SectionIn RO" not in optional and "Section /o" not in optional
    for path in (
        "VST3\\Contrapunk.vst3",
        "CLAP\\Contrapunk.clap",
        "VST3\\Elixir.vst3",
        "CLAP\\Elixir.clap",
    ):
        assert path in source
    uninstall = source[source.index('Section "Uninstall"'):]
    assert uninstall.count("Contrapunk") >= 3 and uninstall.count("Elixir") >= 2


def test_workflow_builds_both_products() -> None:
    source = (ROOT / ".github/workflows/plugin-build.yml").read_text()
    assert source.count("bundle-universal elixir-plugin") == 1
    assert source.count("bundle elixir-plugin") == 1
    assert "productbuild" in source and "Distribution.xml" in source
    assert "Contrapunk-Elixir-Plugins-macOS-universal-bundles.zip" in source
    assert "Contrapunk-Elixir-Plugins-Windows-x64-bundles.zip" in source
    assert "packaging\\windows\\ContrapunkPlugins.nsi" in source


if __name__ == "__main__":
    test_distribution()
    test_macos_removal()
    test_windows_components_and_removal()
    test_workflow_builds_both_products()
    print("installer selection/removal contracts passed")
