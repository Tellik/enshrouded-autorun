# Product modules

Each direct child of `modules/` is a product module with its own behavior and, where needed, its own `ui/` directory. `modules/transport/` is a module too. A window passes work to its owning module and presents the result instead of implementing the same behavior again.

See [code ownership](../docs/development/code-ownership.en.md) for the current repository map and action-to-file guide.

`modules/player-name/` owns live character-name detection from the Enshrouded login log and menu memory, plus persistence of names that have been detected live. The Modloader UI calls it to populate the detected-name setting and reports detection status through the existing logging module.

`modules/game-improvements/` owns optional game-start features and their controls in the shared Settings page. Its native No-Intro action adds Enshrouded's `+com_skipIntroVideo 1` startup option before the bootstrap releases the game entrypoint. It reads its master switch and feature switch from the existing loader configuration and does not load another DLL.
