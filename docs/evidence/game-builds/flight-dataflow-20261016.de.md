# Flugfunktionen: Vom funktionierenden Patcheinstieg zu echten Enginefeldern

Untersucht wurde ausschließlich das aktuelle Image mit SHA-256 `4d6d43ea4a91e71c82d60b045dde932179c99e32e40657efe374688b2933589a`. Es wurde kein Spielprozess verändert und keine Spielfunktion im Spiel aufgerufen.

## Infinite Flying: ursprüngliche Zielwinkelgrenze

Der Toolkit-Einstieg bei `0x26ae0e` ersetzt die untere Pitchgrenze von ungefähr fünf Grad durch `-1.57` Radiant. Der ursprüngliche Zweig gehört zu `actor_rotation` bei `0x269d30`.

| Operand im Original | Reflektierte Herkunft und Verwendung |
| --- | --- |
| Actor `+0x38` | `currentAction.triggerContext.usedItem.itemId.value`, Lookup des tatsächlich benutzten Gleiters |
| ActorInput `+0x18c` und `+0x194` | `desiredLocalMoveInput.x` und `.z`, Pitch-/Yaw-Eingaben |
| PlayerInput `+0x2c8` | `fromClient.movementStickInputMode`, Pitchgeschwindigkeit oder Pitchzielwinkel |
| Summe der Impuls-Y-Anteile | Helper `0x3b1260`, bei positivem Y wird direkt die untere Pitchgrenze gewählt |
| ItemInfo `+0x4d4` | `equipment.gliderConfig.pitchAngleSpeed`, maximale Änderung zum bereits geklemmten Ziel |
| Actor `+0xec4` | `rotationInterpolation.targetOrientation`, berechnete Zielquaternion |

21 private Ausführungen des unveränderten Originalzweigs bestätigen die Rechnung für endliche Werte. Bei positiver Impuls-Y-Komponente ist das Ziel genau die untere Grenze. Ansonsten wird das Ziel auf ungefähr fünf bis 80 Grad begrenzt. Änderungen an den regulären Pitch-Eingaben erreichen in diesem Zweig somit keinen negativen Zielwinkel.

Das nachfolgende Original schreibt die Zielquaternion bei `0x26b204` über `0xf51dc0` nach `Actor.rotationInterpolation.targetOrientation`. Derselbe `actor_rotation`-Aufruf liest diese Daten später für die wirkliche Orientierung erneut. Ein nachträgliches Feldschreiben zwischen Frames ist daher noch kein gleichwertiger Ersatz des bestehenden Eingriffs.

Ein weiterer tatsächlich benannter Berechnungsparameter ist `Movement_RotationMod`. Das Original sucht ihn bei `0x26e694` über den Attributkatalog, liest seinen Float bei `0x26e6c2` und multipliziert bei `0x26e6c7` den endgültigen Interpolationsfaktor damit. Anschließend folgt der Faktor `Actor.rotationInterpolation.speed`. Das ist ein belegter Datenpfad für die Drehungsrechnung. Er ersetzt die feste Pitchgrenze nicht und wird hier nicht als Infinite-Flying-Lösung ausgegeben. Die Runtime kann den Namen aus ihrem kanonischen Attributkatalog auflösen.

`Velocity.velocity` ist ebenfalls ein echter Zustand dieser Physik. Der Gleiterpfad liest ihn bei `0x42b777..0x42b79f` und schreibt das integrierte Ergebnis bei `0x42b9bd..0x42b9c8` zurück. Ein eigener Höhenregler auf diesem Feld wäre eine andere Regelung und wird nicht stillschweigend anstelle der bestehenden Pitchfreigabe eingebaut.

## Fly Mode: vorhandener BuildingFlying-Zustand

Die getrennten Toolkit-Stellen `0x2d7f5c` und `0x2d7ec0` führen in den ursprünglichen `player_control_action_sequence`-Pfad. Dieser prüft `Actor.currentState.IsNearBuildingZone`, schließt Dead und HangGliding aus und verlangt zusätzlich ein Bit aus `BasicPlayerInfos` sowie einen freigeschalteten Skill. Der Skilllookup läuft über `g38_skilltree::SkillTree` und `PlayerSkillsUi.skillUnlockMasks`. Die konkrete statische Skill-ID steht als Untersuchungsbeleg im JSON. Ihr fachlicher Ressourcenname ist hier noch nicht ermittelt.

Die Aktivierung und Deaktivierung laufen über den benannten Eingang `PlayerInputType.BuildingModeFly`. Der entsprechende Enumwert ist 39. Der Originalpfad nutzt den normalen Inputconsumer und ruft zum Aktivieren `0x2bd590`, zum Deaktivieren `0x2be120` auf.

Die Aktivierung erzeugt eine eigene Entity aus `CurrentTransform`, `Actor` und `DynamicLocomotion`. Ihre ID landet in `PlayerState.buildingSparkEntityId`. Bei der normalen Spielerfigur setzt die Aktivierung unter den geprüften Wasserbedingungen das Sitting-Bit. Der vorhandene Systemname `player_control_building_fly_mode` bei `0x2db960` bestätigt den Bezug. Dieses System sucht die Sparkentity anhand der gespeicherten ID, überträgt ihre Steuerung und aktualisiert den Fokus.

### Direkter generischer Dateneinstieg

Der Konstruktor setzt das reflektierte Feld `DynamicLocomotion.state` bei `0x2bdbe0` auf `LocomotionState.BuildingFlying`, Wert 11. Der weitere Pfad ist konkret verbunden:

1. Der originale Statechooser `0x41e030` prüft dieses Feld als erste Bedingung. Wenn es bereits BuildingFlying ist, gibt er unmittelbar wieder BuildingFlying zurück. Der Zustand ist bewusst stabil.
2. Beide Dispatchtabellen von `locomotion_execution` wählen bei BuildingFlying den Originalhelper `0x429790`. Die Engine berechnet damit selbst die Bewegung.
3. `gravity_calculate_actor` erkennt BuildingFlying und setzt anschließend `Gravity.isActive` selbst auf false. Die Engine führt den getrackten Gravity-Schreibzugriff in ihrer eigenen Ausführungsphase aus.

Zwei private Ausführungen des Statechoosers und sechs private Ausführungen des Gravityzweigs bestätigen diese Übergänge. Es handelt sich um unveränderte Originalanweisungen mit privaten Daten. Der Lua-Livetest steht aus.

`DynamicLocomotion` besitzt im live geprüften Originalregister weder A- noch B-Trackingbit. Der eigene Dateneingang kann daher die gemeinsame `untracked-components`-Richtlinie verwenden. Dafür ist kein eigener Gravitypatch, Velocityregler oder Aufruf eines Fly-Adapters nötig. Noch offen ist, ob das alleinige Setzen dieses Zustands auf der normalen Spielerfigur vollständige dreidimensionale Steuerung, gewünschtes Kollisionsverhalten und eine saubere Wiederherstellung ergibt. Es ist außerdem kein Nachweis derselben Gleiter-Pitchwirkung wie Infinite Flying.

Ein regulärer API-Request über den BuildingModeFly-Input könnte stattdessen den vorhandenen Sparkmodus unter seinen ursprünglichen Voraussetzungen auslösen. Die Toolkit-Permissionumgehung ist damit nicht automatisch übertragen. `PlayerState` und `PlayerSkillsUi` sind A-getrackt und werden nicht durch die aktuelle Schreibfreigabe pauschal geöffnet.

## Build Flight: Gravitationseingriff

Die Toolkit-Stelle `0x29e6cf` liegt in `gravity_apply_velocity_actor`. Das Original integriert dort `Velocity.velocity.y`. Das ist kein Schalter für den ursprünglichen BuildingFlying-Zustand. Auch die vorhandenen `PlayerLocomotion.gravityAscending` und `gravityDescending` sind berechnete Zwischenwerte. Ihr Produzent kann sie vor dem Verbraucher erneut setzen. Ohne Prüfung der gesamten Producer-/Consumerkette wird ihr Überschreiben deshalb nicht als gleichwertiger Ersatz freigegeben.

## Reproduzierbare Belege

- `infinite-flight-operands-20261016.json` enthält Feldpfade, Instruktionsslices und 21 Originalfälle der Pitchklemmung.
- `building-flight-state-20261016.json` enthält die ursprünglichen State-/Gravity-/Movementfunktionen, den Aktivierungspfad und acht erfolgreiche Originalfälle.
- `trace-infinite-flight-operands.py` und `trace-building-flight-state.py` erzeugen diese Belege ausschließlich aus Dateidaten und privater Unicorn-Emulation.

| KI-Codex-Check | Ergebnis |
| --- | --- |
| Einstieg | Vorhandene funktionierende Toolkit-Patchstellen verfolgt |
| Datenfluss | Eingaben, Zielquaternion, Attributfaktor, originaler BuildingFlying-Zustand und Gravityverbrauch verbunden |
| Prüfung | 29 private Originalfälle erfolgreich, keine Spielwrites |
| Offen | Echter Lua-Flugtest, vollständige Steuerung und Restore. 1:1-Gleiter-Pitchersatz weiterhin unbewiesen |
