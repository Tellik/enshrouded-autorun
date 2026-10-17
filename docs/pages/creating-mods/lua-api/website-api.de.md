---
{
  "id": "website-api",
  "locale": "de",
  "title": "Durchsuchbare API-Referenz",
  "summary": "Die vorhandene durchsuchbare Lua-, Typen- und Ressourcensuche.",
  "view": "api",
  "navigation": {
    "path": [
      "MODDING"
    ],
    "label": "Modding-API"
  }
}
---


    <div class='page-intro'><div class='eyebrow'>API-REFERENZ</div><h1>Funktionen finden<br>und im Mod verwenden.</h1><p class='lead'>Suche nach einem Funktionsnamen oder beschreibe, was dein Mod tun soll. Die Treffer zeigen dir Zweck, Eingaben und Ergebnis. Für typische Aufgaben findest du direkt eine Schritt-für-Schritt-Anleitung.</p></div>
    <div class='callout info'><b>So verwendest du die Suche</b><span>Wähle EML oder ShroudForge, wenn du die Treffer eingrenzen willst. Klicke auf eine Zeile, um die Erklärung zu öffnen. Dort findest du Signatur, Parameter, Rückgabe und passende Anleitungen. Ein Lua-Beispiel kannst du direkt kopieren.</span></div>
    <div class='api-examples'><span>Häufige Aufgaben</span><button type='button' data-api-query='shroudforge.notifications.publish'>Nachricht im Modloader zeigen</button><button type='button' data-api-query='shroudforge.settings.get'>Mod-Einstellung lesen</button><button type='button' data-api-query='shroudforge.ui.on_action'>Modloader-Knopf mit Lua verbinden</button><button type='button' data-api-query='runtime.network.send_mod'>Nachricht zwischen Mod-Prozessen senden</button><button type='button' data-api-query='shroudforge.log.info'>Mod-Meldung schreiben</button></div>
    <div class='catalog-meta'><span class='status-dot'></span><span id='build-state'>Lua-API wird geladen …</span><span class='catalog-version' id='catalog-version'></span></div>
    <div class='catalog-tabs'><button class='active' data-catalog='api'>Lua-Funktionen</button><button data-catalog='types'>Spiel-Details</button><button data-catalog='resources'>Spieldateien</button></div>
    <section id='catalog-api' class='catalog-section'><div class='catalog-tools'><div class='catalog-switch'><button class='active' data-api-source='all'>Alle</button><button data-api-source='eml'>EML</button><button data-api-source='sf'>ShroudForge</button></div><input id='api-search' type='search' aria-label='Lua-API durchsuchen' placeholder='Aufgabe oder Funktionsname suchen…'></div><div id='api-result-count' class='api-result-count' aria-live='polite'></div><div id='api-results' class='catalog-results'></div></section>
    <section id='catalog-types' class='catalog-section hidden'><div class='catalog-tools'><p class='muted'>Begriffe und Werte aus einer bestimmten Enshrouded-Version finden.</p><input id='type-search' type='search' placeholder='Name oder Feld suchen…'></div><div id='type-results' class='catalog-results'></div></section>
    <section id='catalog-resources' class='catalog-section hidden'><div class='catalog-tools'><p class='muted'>Namen und Kennnummern der Spieldateien finden.</p><input id='resource-search' type='search' placeholder='Art oder Kennnummer suchen…'></div><div id='resource-results' class='catalog-results'></div></section>
    <div class='callout warning'><b>Daten zur angezeigten Spielversion</b><span>Die Spielinformationen gehören zu einer bestimmten Version und können sich nach einem Update ändern.</span></div>
  