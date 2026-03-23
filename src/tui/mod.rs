// Terminal UI entrypoint.
// Built with ratatui. Vim-inspired keybindings.
//
// Three main views:
//   RfdList       — browse and search RFDs
//   RfdDetail     — read a rendered RFD
//   Discussions   — threads and messages for the current RFD
//
// A background tokio task maintains a WebSocket connection to the server
// and sends WsEvents through a channel to the render loop.

pub mod views;

use anyhow::Result;

pub async fn run() -> Result<()> {
    // TODO: load config (server address, token from ~/.config/parley/token)
    // TODO: init terminal (ratatui setup)
    // TODO: spawn background WebSocket task → mpsc channel into event loop
    // TODO: enter main event loop:
    //   - poll crossterm events (keyboard input)
    //   - poll ws_rx channel (server push events)
    //   - render current view
    // TODO: restore terminal on exit
    todo!()
}
