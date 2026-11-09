//! `intracraft-assets [FOLDER]`: writes the pallet and load models as glTF files into FOLDER, by default the viewer's
//! `assets/pallets` (ADR-0026).

use std::process::ExitCode;

use intracraft_library::reference::Reference;

fn main() -> ExitCode {
    let folder = std::env::args().nth(1).map(std::path::PathBuf::from).unwrap_or_else(|| {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../apps/intracraft-viewer/assets").join(intracraft_assets::FOLDER)
    });
    let reference = match Reference::built_in() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("intracraft-assets: the reference data doesn't read: {e}");
            return ExitCode::FAILURE;
        }
    };
    if let Err(e) = std::fs::create_dir_all(&folder) {
        eprintln!("intracraft-assets: {}: {e}", folder.display());
        return ExitCode::FAILURE;
    }
    for (name, bytes) in intracraft_assets::files(&reference) {
        let path = folder.join(&name);
        if let Err(e) = std::fs::write(&path, &bytes) {
            eprintln!("intracraft-assets: {}: {e}", path.display());
            return ExitCode::FAILURE;
        }
        println!("{} ({} kB)", path.display(), bytes.len().div_ceil(1024));
    }
    ExitCode::SUCCESS
}
