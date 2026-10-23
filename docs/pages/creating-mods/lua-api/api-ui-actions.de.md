---
{
  "id": "api-ui-actions",
  "locale": "de",
  "title": "Modloader-Aktionsknöpfe mit Lua verbinden",
  "summary": "Eine Aktions-ID in der Moddatei verbindet den sichtbaren Knopf mit einer Callback-Funktion deines Mods.",
  "navigation": {
    "path": [
      "Lua-API-Anleitungen",
      "Funktionsanleitungen"
    ]
  },
  "apiSymbols": [
    "shroudforge.ui.on_action",
    "shroudforge.ui.toggle_modloader"
  ]
}
---

# Modloader-Aktionsknöpfe mit Lua verbinden

Eine Aktion steht unter `groups[].actions[]`. Die `id` wird an `shroudforge.ui.on_action` übergeben. Beide IDs müssen gleich geschrieben sein.

```lua
shroudforge.ui.on_action("resetFeature", function()
  shroudforge.log.info("Reset action requested")
end)
```

Der Mod braucht `runtime`. Die Funktion muss registriert sein, während die Mod-Laufzeit geladen wird. `label` und `style` beschreiben die sichtbare Schaltfläche, `confirm` ist eine optionale Rückfrage.

Die Aktion selbst führt nur deinen Callback aus. Sie garantiert keine erfolgreiche Spieländerung. Prüfe Rückgabewerte und logge das Ergebnis. Details der API stehen in der [API-Referenz](#api).

Ein aktiver Client-Mod kann die Modloader-Oberfläche über denselben Toggle wie den konfigurierten Hotkey öffnen oder schließen:

```lua
shroudforge.ui.toggle_modloader()
```

Der Aufruf signalisiert direkt den UI-Prozess. Er simuliert keinen Tastendruck. Dafür muss die Modloader-Oberfläche für die laufende Spielsitzung gestartet sein.

Unter **Einstellungen → Allgemein → Darstellung → Mod-Menü im Spiel anzeigen** lässt sich der native Eintrag **Mods** im Enshrouded-Hauptmenü ein- und ausblenden. Er verwendet die Schaltflächen des Spiels und denselben Modloader-Toggle wie diese Lua-Funktion und der konfigurierte Hotkey. Eine Änderung der Checkbox gilt beim nächsten Aufbau des Hauptmenüs. Die native Anbindung wird beim Spielstart eingerichtet. Nach einem ShroudForge-Update ist dafür ein Spielneustart erforderlich.
