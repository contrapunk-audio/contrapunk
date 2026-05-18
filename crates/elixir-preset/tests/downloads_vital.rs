//! Local/manual validation against the user's Downloads folder.
//!
//! Ignored by default because CI/dev machines won't have these files.

use std::path::PathBuf;

use elixir_preset::{import_vital_bank_file, import_vital_file};

#[test]
#[ignore = "requires ~/Downloads Vital assets"]
fn imports_current_downloads_vital_assets() {
    let home = std::env::var("HOME").expect("HOME");
    let downloads = PathBuf::from(home).join("Downloads");
    let individual = [
        "Dear April Pad Preset.vital",
        "Cyberpunk 2077 Preset.vital",
        "Angular Keys Presets.vital",
        "Most Liked Preset FLOAT_KEYS_I.vital",
        "Particle Arts Lead Preset.vital",
        "Lofi Keys Preset.vital",
    ];
    for name in individual {
        let path = downloads.join(name);
        let preset =
            import_vital_file(&path).unwrap_or_else(|err| panic!("{}: {err}", path.display()));
        assert!(
            !preset.name.trim().is_empty(),
            "{} imported empty name",
            path.display()
        );
        assert!(
            preset.patch.chorus_mix.is_some()
                || preset.patch.delay_mix.is_some()
                || preset.patch.reverb_mix.is_some()
                || preset.patch.filter_cutoff.is_some()
        );
    }

    let bank =
        import_vital_bank_file(downloads.join("Vital Account.vitalbank")).expect("bank import");
    assert!(
        bank.presets.len() >= 75,
        "expected at least 75 presets, got {}",
        bank.presets.len()
    );
    assert!(
        bank.wavetable_paths.len() >= 21,
        "expected factory wavetable paths, got {}",
        bank.wavetable_paths.len()
    );
    assert!(
        bank.skipped_entries.is_empty(),
        "skipped entries: {:?}",
        bank.skipped_entries
    );
}
