# World Editor mod

This package restores the World Editor from the 0.9 reference with its existing
settings, SFBP V7 files, editor actions, library, rotation, paste/undo journal,
and Steam P2P protocol. The Lua code uses the public `runtime.world` and asset
APIs. It contains no game-code patches, fixed game RVAs, or function-hash lookups.

**Enshrouded 1.0 migration is in progress.** The experimental voxel and entity
backends that used fixed function addresses and per-build profile additions were
removed. World Editor's session-bound voxel reader now resolves the live
`ecs::VoxelGrid` and `VoxelWorld` registrations by type name and calls their
original registered read interface. It stages the full result and publishes it
only after the world session is checked again. The Lua read path no longer falls
back to the profile-based voxel function. Native fixtures cover success, pending,
bad session, and read-only cases. The updated runtime and mod are installed in
the test game, but the F8 canary has not yet been repeated. Existing scene-owner
resolution remains profile-backed. Voxel writing, prop creation/removal, and a
complete 1.0 paste/undo cycle still need native APIs. Collision, replication,
and save persistence remain unverified.
See the [dated 1.0 investigation](../../docs/evidence/runtime/2026-10-19-world-editor-1.0.de.md)
for the exact executable identity, reader contract, rejected experiment and
remaining contracts.

The following describes the preserved editor workflow and required runtime
contract. Statements about native writes describe the checks the mod requests,
not a new live verification of the Enshrouded 1.0 backend. See the current
[runtime guide](../../docs/runtime/native-runtime-and-game-profiles.en.md) and
[multiplayer guide](../../docs/modding/multiplayer-and-target-processes.de.md).

Read-only inspection of the installed 1.0 build `1269579` through the KFC4
parser found all ItemInfo field paths used by this mod. The preserved recipe
filter accepts 1,774 distinct placement recipes from 4,262 ItemInfo resources.
This verifies the file-resource contract, not loaded-world access or placement.

The client automatically uses the native direct path in a validated local
world and Steam P2P in a validated read-only server world. While the native
world context is unknown, F7 and F4 pause without sending a local write or a
P2P request. Install and enable World Editor on both the client and the
Dedicated Server. The server runs the same mod headlessly. it has no UI or
keyboard hook. The client sends the existing SFBP V7 text through
`runtime.network.send_mod`, and the server parses it and invokes the same native
paste, snapshot, verification, and undo journal used by direct edits.

The loader's general **Network** module establishes the Steam session without
World Editor specific setup. It accepts incoming Steam session requests through
the game's callback pump and, on Dedicated Servers, sends a health probe every
five seconds to currently connected Enshrouded players. A client learns the
remote server SteamID64 from that probe. the remote-server fallback and
optional authorized-client list live in Settings > Modules > Network. An empty
authorized-client list uses the server's authenticated connected-player list,
read from `logs/enshrouded_server.log`. The World Editor uses this shared
service and keeps only its blueprint transfer, validation, authorization, and
undo protocol.
The P2P transfer limit is 32 MiB per
blueprint. local blueprint files keep their existing 256 MiB limit. Capturing
and saving remain on the client. F7 sends the same V7 blueprint body to the
server. F4 sends the server-issued undo token back, so process-local entity
handles never cross the network. Switching between local and server worlds
changes the route automatically. The dedicated server keeps one in-memory undo
journal. restarting it discards that journal. If the client loses a paste
reply while the server retains an unresolved journal, a later paste refusal
returns that peer's existing undo token so F4 can resume recovery. A stale
client undo token cannot permanently block F7: the client may submit the next
paste, and the server accepts it only after its own prior undo journal is
complete. If an undo is still active, the server keeps rejecting edits until
it reports completion.

The P2P transport retains SFBP validation, authorization, and duplicate-request
guards from the reference implementation. **Live dedicated-server replication
and save persistence still require an in-game canary test.** A confirmed native readback
means the server runtime observed its world write. another client seeing the
change and the world retaining it after rejoin are separate checks. See the
[dated multiplayer investigation](../../docs/evidence/runtime/2026-10-09-world-editor-multiplayer.de.md)
for historical evidence only, not current 1.0 support.

This independent Lua mod implements its editor, blueprint format, selection, rotation, preview plan, paste, and undo behavior itself. It uses ShroudForge's public Lua APIs only for game/runtime access: `runtime.world.cursor.get`, native prop recipe registration and bounds queries, voxel operations, entity operations, asset reads, settings, logging, and export storage. Native engine access stays behind those APIs. the mod has no private loader hook or built-in editor implementation.

While the game window is focused, the World Editor polls **F3–F8**: F3 rotates the active blueprint by 90° around its stored up axis, F4 undo, F5 mark selection corner A/B, F6 resets editor state and offers server release during a stalled or incomplete remote operation, F7 pastes at the live cursor, and F8 captures, saves, and selects a blueprint. After a server operation stops reporting progress for 10 seconds, its progress state turns red and the editor message explains that F6 can abandon it. The client remains locked from F7 until the server confirms release. **F1 remains available to Enshrouded.** The dashed **New blueprint** tile stays at the far left. Starting a new capture with F5 automatically makes that tile active and disarms the prior blueprint. after F8, the new saved blueprint becomes active. Selecting any saved blueprint makes it ready for F7 immediately. F6 is not required to switch blueprints. The active card has a persistent gold border and an **ACTIVE** badge, while a card being loaded has a separate blue **LOADING** state. The library opens as a single row. the chevron expands it downward, showing up to six rows before vertical scrolling reveals additional rows. Selecting an item collapses the grid. The most recently selected or created blueprint appears first after the permanent New blueprint tile. Double-click or use the card’s **Edit** button to manage it. The manager can browse a screenshot folder, select a screenshot, and apply it as the blueprint cover. The screenshot folder is remembered. common Enshrouded screenshot folders are detected on first use. The **Settings** button opens the Modloader UI at the World Editor module settings. **F2 opens or hides the single World Editor window**. the red **×** button also hides it. **F9 remains the Modloader UI shortcut. F10 remains the Debug Console shortcut.** The same editor actions are available as buttons in the mod settings.

When the game starts, ShroudForge launches one World Editor window host using the same Tao/Wry window approach as the Modloader UI and Debug Console. The fixed-size window is centered lower in the game HUD. Its instruction line is at the top and the blueprint cards are arranged underneath. F2 only shows or hides this whole window. other function keys perform their editor actions without changing window visibility. A single click loads/selects a blueprint and a double-click opens the blueprint manager. The manager discovers common Enshrouded screenshot folders, lets the user browse another folder, previews supported images, and copies the chosen image into the blueprint library as `.png` plus a thumbnail. It never modifies the original screenshot. New or copied `.sfbp` files placed in `exports/world-editor/blueprints` are discovered automatically. To start a capture, select the building hammer, choose a single voxel, aim at the first corner and press F5. aim at the opposite corner and press F5 again, then press F8. The mod does not attempt to inspect or select the hammer or voxel mode automatically.

The persistent `.sfbp` format writes and accepts `SHROUDFORGE_WORLD_BLUEPRINT_V7`, with either a voxel channel plus props or props alone. Each voxel cell stores its value and ShroudEdit-compatible coverage (`Unknown`, `Empty`, or `Occupied`). Captures classify successfully read non-zero cells as occupied and zero cells as empty, preserving the reference format's coverage convention. On paste, unknown cells preserve the target, empty cells clear it in replace mode, and additive mode skips empty cells. The file also stores the local selection extent and voxel-channel offset from the selection anchor, along with item IDs, native entity-template UUIDs, prop transforms, and the blueprint up axis. On paste, the cursor world position is the anchor, as in ShroudEdit. Voxel pastes are refused when the resulting channel origin is not aligned to the active grid. Props-only blueprints carry no voxel channel. Spawn requests the captured native template UUID and ItemInfo tracking recipe through the public runtime API and requires verified native support for the active build. A confirmed spawn must return the actual opaque live entity handle. Transform lookup, target replacement, and undo use that handle through native APIs. Blueprints store no build-specific runtime entity address.

The Lua voxel contract limits each read or write to 65,536 cells and requests the `voxel` channel's grid origin and cell size from the runtime. The current session-bound reader resolves the named grid services and calls the original registered interface roles for origin, readiness, and cell reads. It uses the original 0.5 m grid, validates every tile task for the requested region, and stages the complete result before returning it to Lua. Fixture tests cover complete reads, pending reads, and session changes. The in-game F8 canary remains open. Voxel writing, collision, and save persistence require separate implementation or verification. Prop capture registers `ItemInfo` placement AABBs through the Lua API, then the dedicated native world query scans the live entity snapshot and applies rotated, scaled recipe bounds before returning matching prop handles and transforms. Transform lookup and scale updates use the current shared ECS implementation. Scale updates compare the expected `CurrentTransform` snapshot and change only its scale fields through the canonical guarded ECS writer, including its ownership checks, scene gates, tracking restrictions, verification, and rollback. The mod uses the world API and does not provide a second component writer.

A failed transform read alone never confirms removal. Only `nil, "entity-absent"` from the session-bound original membership check means that the issued handle's identity is absent. Undo and replacement retain their journals when a snapshot is unreadable, a session changes, or a write outcome is uncertain. The original prop is recorded before a removal attempt, so an unavailable readback cannot lose its recovery entry or justify spawning a duplicate.

Undo reads the target voxels before restoring the saved snapshot and pauses if those cells changed after paste. It checks each pasted prop's live handle and transform before removing it. On a paste failure, the mod immediately attempts to restore the pre-paste voxel and prop snapshot, following ShroudEdit's rollback path. If a native operation cannot be verified, it retains recovery state for F4 retry. For a dedicated-server operation with no progress for 10 seconds, the client shows that the server may be stuck and offers F6 to abandon the operation. F6 sends an authenticated, repeatable release request; the client keeps F7 locked until the server confirms it has discarded that operation's undo journal and released its lock. Press F6 again to resend if confirmation is lost. Abandoning may leave partial terrain or props in the world, and the old operation can no longer be undone. After confirmation, another World Editor operation can start. If the server is unreachable, F7 remains paused until it receives and confirms the release request. F7 uses the live cursor position for its target. In the Modloader UI (F9), open **World Editor**:

- Set **Blueprint up axis** to X, Y, or Z before capturing. The axis is stored in the blueprint. changing this setting later does not change an already captured or loaded blueprint.
- Set **Initial paste rotation** to 0, 1, 2, or 3 quarter turns (0°, 90°, 180°, 270°), then press **F3** in game to advance the active blueprint by 90° each time. The Modloader's **Rotate blueprint (F3)** button does the same. Rotation cycles through 0°, 90°, 180°, and 270° before returning to 0°. it applies to preview and paste.
- Click **Preview placement plan** to resolve the live cursor target and report the rotated dimensions and counts in the ShroudForge log. This preview is non-destructive, but it is a plan preview only: it does not draw a 3D ghost or show the schematic in the game world.
- Use **Paste active blueprint** or F7 only after checking the plan. Those actions write to the world.

Mark selection corners A and B, then use **Capture props only from A/B selection** to create a blueprint without a voxel channel. Use **Capture voxels and props** or F8 for a combined capture. Props-only blueprints can be pasted at the live cursor, a marked target, or the **Props-only target world X/Y/Z** settings without reading voxel grid metadata. Choose whether to replace or add voxel cells and whether to keep or replace intersecting target props. Replacing target props is refused when their recipe, native template identity, or transform cannot be safely verified. F8 names new persistent combined captures `capture-1`, `capture-2`, and so on, without replacing an existing export.

Set **Maximum props per blueprint** in the mod settings to control the per-blueprint prop count (default 60,000. supported range 1–1,000,000). Capture, save, load, and paste all enforce the current value. captures above it are refused instead of silently truncated.

Prop capture scans incrementally: each native scan slice checks at most 8,192 entity pointers and yields after approximately 2 ms of scan time. Initialization, final result validation, and individual memory reads are outside a hard real-time guarantee. Lua continues pending scans on subsequent updates. Incomplete results never become a saved blueprint. A changed ECS epoch, unavailable world, 30 seconds without native scan progress, or the 120-second overall scan deadline aborts capture. F6 cancels a local pending editor callback. For a remote server edit, F6 requests an authenticated server release and waits for confirmation. Recipe loading and voxel/file operations have separate execution paths. Capture stage messages in the log distinguish recipe resolution, pending enumeration, and completion.

The standalone mod implements its editor and V7 blueprint format itself. Direct world operations use the registered native Lua APIs. In multiplayer, the server executes those same APIs. the client never writes a replicated read-only grid or sends runtime entity handles. Runtime behavior that depends on the live game, including collision, replication and save persistence, requires in-game verification.

The World Editor header shows the active operation and its current phase for capture/save, load, library refresh/rename/duplicate/delete, preview, placement, rollback, and undo. It reports a step count only when the operation has a known number of coarse phases. otherwise the bar is indeterminate and shows elapsed time. Progress and visible UI messages are written through the shared logger to the role-appropriate ShroudForge log under the `world-editor` source. there is no separate UI log. INFO records meaningful phase changes and confirmed results, WARN records blocked or uncertain work, ERROR records failed writes or verification, DEBUG records inputs, target details, readiness changes, and window visibility, and TRACE records deduplicated full UI snapshots. See the current [logging API](../../docs/pages/creating-mods/lua-api/api-logging.en.md) for new messages. In multiplayer the server logs its own native phases and sends phase updates over the existing P2P request so the client can show them and record the received server status in its own World Editor log. Live readback confirms the in-world result. it does not confirm save persistence.
