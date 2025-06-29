use crate::directory::IwadDirs;
use software_registry::LocalSoftwareRegistry;

mod software_registry;

static UNINSTALLER_STRING: &str = "\\uninstl.exe /S ";

pub fn add_dirs(iwad_dirs: &mut IwadDirs) {
    check_uninstall_strings(iwad_dirs);
    check_install_root_paths(iwad_dirs);
    check_steam_edition(iwad_dirs);
    check_dos_defaults(iwad_dirs);
}

/// Check for the uninstallation strings from the CD versions
#[inline]
fn check_uninstall_strings(iwad_dirs: &mut IwadDirs) {
    #[inline]
    fn build_uninstall_registry(game: &str) -> LocalSoftwareRegistry {
        LocalSoftwareRegistry::new(
            &format!("Microsoft\\Windows\\CurrentVersion\\Uninstall\\{game}"),
            "UninstallString",
        )
    }

    let uninstall_registries = vec![
        build_uninstall_registry("Ultimate Doom for Windows 95"),
        build_uninstall_registry("Doom II for Windows 95"),
        build_uninstall_registry("Final Doom for Windows 95"),
        build_uninstall_registry("Doom Shareware for Windows 9"),
    ];

    for registry in &uninstall_registries {
        let Some(val) = registry.query_value() else {
            continue;
        };
        if let Some(i) = val.find(UNINSTALLER_STRING) {
            let i = i + UNINSTALLER_STRING.len();
            iwad_dirs.add_dir(&val[i..]);
        }
    }
}

/// Check for GOG.com and Doom: Collector's Edition
#[inline]
fn check_install_root_paths(iwad_dirs: &mut IwadDirs) {
    let install_registries = vec![
        // Doom Collector's Edition
        LocalSoftwareRegistry::new("Activision\\DOOM Collector's Edition\\v1.0", "INSTALLPATH"),
        // Doom II
        LocalSoftwareRegistry::new("GOG.com\\Games\\1435848814", "PATH"),
        // Doom 3: BFG Edition
        LocalSoftwareRegistry::new("GOG.com\\Games\\1135892318", "PATH"),
        // Final Doom
        LocalSoftwareRegistry::new("GOG.com\\Games\\1435848742", "PATH"),
        // Ultimate Doom
        LocalSoftwareRegistry::new("GOG.com\\Games\\1435827232", "PATH"),
    ];

    let install_registries_subdirs = vec![
        ".",
        "Doom2",
        "Final Doom",
        "Ultimate Doom",
        "Plutonia",
        "TNT",
        "base\\wads",
    ];

    for registry in &install_registries {
        let Some(install_dir) = registry.query_value() else {
            continue;
        };
        for subdir in &install_registries_subdirs {
            let dir = format!("{install_dir}\\{subdir}");
            iwad_dirs.add_dir(&dir);
        }
    }
}

/// Check for Doom downloaded via Steam
#[inline]
fn check_steam_edition(iwad_dirs: &mut IwadDirs) {
    let steam_subdirs = vec![
        "steamapps\\common\\doom 2\\base",
        "steamapps\\common\\doom 2\\finaldoombase",
        "steamapps\\common\\final doom\\base",
        "steamapps\\common\\ultimate doom\\base",
        // From Doom 3: BFG Edition:
        "steamapps\\common\\DOOM 3 BFG Edition\\base\\wads",
    ];
    let reg = LocalSoftwareRegistry::new("Valve\\Steam", "InstallPath");
    let Some(base_dir) = reg.query_value() else {
        return;
    };

    for subdir in steam_subdirs {
        let dir = format!("{base_dir}\\{subdir}");
        iwad_dirs.add_dir(&dir);
    }
}

/// Default install directories for DOS Doom
#[inline]
fn check_dos_defaults(iwad_dirs: &mut IwadDirs) {
    // The directories are relative to root drive, for instance
    // "\\doom2" should be equivalent to "C:\\doom2".

    // These are the default install directories used by the deice
    // installer program:

    // Doom II
    iwad_dirs.add_dir("\\doom2");

    // Final Doom
    iwad_dirs.add_dir("\\plutonia");
    iwad_dirs.add_dir("\\tnt");

    // Ultimate Doom
    iwad_dirs.add_dir("\\doom_se");

    // Shareware / Registered Doom
    iwad_dirs.add_dir("\\doom");
    iwad_dirs.add_dir("\\dooms");

    // Shareware versions
    iwad_dirs.add_dir("\\doomsw");
}
