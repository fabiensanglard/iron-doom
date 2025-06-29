use bevy::app::plugin_group;

plugin_group! {
    #[derive(Debug)]
    pub struct CratesPlugins {
        bevy::input:::InputPlugin,
        bevy::log:::LogPlugin,
        bevy::time:::TimePlugin,
        bevy::state::app:::StatesPlugin,
        cli:::CliPlugin,
        exit:::ExitPlugin,
        game_state:::GameStatePlugin,
        level:::LevelPlugin,
        rand:::RandPlugin,
        screen_melt:::ScreenMeltPlugin,
        title_screen:::TitleScreenPlugin,
        wad:::WadPlugin,
        window:::WindowPlugin,
        world_view:::WorldViewPlugin,
    }
}
