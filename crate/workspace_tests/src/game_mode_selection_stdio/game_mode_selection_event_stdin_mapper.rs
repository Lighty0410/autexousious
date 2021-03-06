#[cfg(test)]
mod tests {
    use game_mode_selection_model::{GameModeIndex, GameModeSelectionEventArgs};
    use menu_model::MenuEvent;
    use stdio_spi::StdinMapper;

    use game_mode_selection_stdio::GameModeSelectionEventStdinMapper;

    #[test]
    fn maps_select_event() {
        let args = GameModeSelectionEventArgs::Select {
            index: GameModeIndex::StartGame,
        };

        let result = GameModeSelectionEventStdinMapper::map(&(), args);

        assert!(result.is_ok());
        assert_eq!(MenuEvent::Select(GameModeIndex::StartGame), result.unwrap())
    }

    #[test]
    fn maps_close_event() {
        let args = GameModeSelectionEventArgs::Close;

        let result = GameModeSelectionEventStdinMapper::map(&(), args);

        assert!(result.is_ok());
        assert_eq!(MenuEvent::Close, result.unwrap())
    }
}
