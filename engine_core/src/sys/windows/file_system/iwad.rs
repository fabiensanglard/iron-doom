use software_registry::LocalSoftwareRegistry;

use crate::file_system::IwadDirs;

mod software_registry;

static UNINSTALLER_STRING: &str = "\\uninstl.exe /S ";
static STEAM_BFG_GUS_PATCHES: &str =
    "steamapps\\common\\DOOM 3 BFG Edition\\base\\classicmusic\\instruments";

pub fn add_iwad_dirs(iwad_dirs: &mut IwadDirs) {
    check_uninstall_strings(iwad_dirs);
    check_install_root_paths(iwad_dirs);
    check_steam_edition(iwad_dirs);
    check_dos_defaults(iwad_dirs);
    check_steam_gus_patches();
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

    for reg in &uninstall_registries {
        let val = match reg.query_value() {
            None => continue,
            Some(val) => val,
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
        // Strife: Veteran Edition
        LocalSoftwareRegistry::new("GOG.com\\Games\\1432899949", "PATH"),
        // Heretic
        LocalSoftwareRegistry::new("GOG.com\\Games\\1290366318", "PATH"),
        // Hexen
        LocalSoftwareRegistry::new("GOG.com\\Games\\1247951670", "PATH"),
        // Hexen: Deathkings of a Dark Citadel
        LocalSoftwareRegistry::new("GOG.com\\Games\\1983497091", "PATH"),
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

    for reg in &install_registries {
        let install_dir = match reg.query_value() {
            None => continue,
            Some(val) => val,
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
        "steamapps\\common\\heretic shadow of the serpent riders\\base",
        "steamapps\\common\\hexen\\base",
        "steamapps\\common\\hexen deathkings of the dark citadel\\base",
        // From Doom 3: BFG Edition:
        "steamapps\\common\\DOOM 3 BFG Edition\\base\\wads",
        // From Strife: Veteran Edition:
        "steamapps\\common\\Strife",
    ];
    let reg = LocalSoftwareRegistry::new("Valve\\Steam", "InstallPath");
    let base_dir = match reg.query_value() {
        None => return,
        Some(dir) => dir,
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

    iwad_dirs.add_dir("\\doom2"); // Doom II
    iwad_dirs.add_dir("\\plutonia"); // Final Doom
    iwad_dirs.add_dir("\\tnt");
    iwad_dirs.add_dir("\\doom_se"); // Ultimate Doom
    iwad_dirs.add_dir("\\doom"); // Shareware / Registered Doom
    iwad_dirs.add_dir("\\dooms"); // Shareware versions
    iwad_dirs.add_dir("\\doomsw");

    iwad_dirs.add_dir("\\heretic"); // Heretic
    iwad_dirs.add_dir("\\hrtic_se"); // Heretic Shareware from Quake disc

    iwad_dirs.add_dir("\\hexen"); // Hexen
    iwad_dirs.add_dir("\\hexendk"); // Hexen Deathkings of the Dark Citadel

    iwad_dirs.add_dir("\\strife"); // Strife
}

/// The BFG edition ships with a full set of GUS patches. If we find them,
/// we can autoconfigure to use them.
#[inline]
fn check_steam_gus_patches() {
    // ATTENTION! - MISSING DOOM IMPLEMENTATION: d_iwad.c - CheckSteamGUSPatches

    let reg = LocalSoftwareRegistry::new("Valve\\Steam", "InstallPath");
    let base_dir = match reg.query_value() {
        None => return,
        Some(dir) => dir,
    };

    // Already configured? Don't stomp on the user's choices.
    // current_path = M_GetStringVariable("gus_patch_path");
    // if (current_path != NULL && strlen(current_path) > 0)
    // {
    //     return;
    // }

    let patch_dir = format!("{base_dir}\\{STEAM_BFG_GUS_PATCHES}");
    let _test_patch_dir = format!("{patch_dir}\\ACBASS.PAT");

    // Does acbass.pat exist? If so, then set gus_patch_path.
    // if (M_FileExists(test_patch_path))
    // {
    //     M_SetVariable("gus_patch_path", patch_path);
    // }
}
