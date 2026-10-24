# Autorun

This client mod is the first movement proof of concept for Enshrouded Steam
build `26082781`. F8 toggles ordinary forward movement. Press F8 again or S
to stop. The keys are configurable in the mod settings. The mod is disabled by
default while gameplay verification is pending.

The native request lasts at most 500 ms without renewal. It stops on backward
movement, focus or world-session loss, missing local input, or mod unload. The
hook accepts only the verified local `ClientPlayerInput` in this exact executable.
It writes the forward stick after physical input is collected; left/right input
still steers, and the stick is kept at unit magnitude rather than requesting
sprint or the game's slower Walk action.

The first test should take place in a safe, open area:
wait for the world to settle, tap F8 once, verify forward movement and steering,
tap F8 again to stop, then re-enable and press S to cancel. The mod logs native
hook, local match, and write counts. This basic movement/toggle/S test worked
under Proton in single-player and when joining a dedicated server with only
the client mod installed. Menus, focus loss, death, travel, native Windows,
and multiplayer sessions with other active players have not yet been verified.
