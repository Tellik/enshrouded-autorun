---
{
  "id": "website-server-windows",
  "locale": "de",
  "title": "Server unter Windows starten",
  "summary": "Vorhandene Schritt-für-Schritt-Anleitung für Windows-Server.",
  "view": "server-windows",
  "navigation": {
    "path": [
      "QUICKSTART"
    ],
    "label": "Windows Dedicated Server"
  }
}
---


    <div class="page-intro"><div class="eyebrow">QUICKSTART · MULTIPLAYER · WINDOWS</div><h1>Windows Dedicated<br>Server.</h1><p class="lead">Erledige zuerst <a href="#server-files" data-view="server-files">ShroudForge-Dateien auf dem Server installieren</a>. Wähle danach den Serverordner aus und aktiviere den Mod.</p></div>
    <section class="server-docker-guide"><article><h3>Server-Mod-Einstellungen öffnen</h3><p>Öffne auf deinem Windows-PC PowerShell im Ordner mit den ShroudForge-Dateien. Führe diesen Befehl aus und ersetze den Beispielpfad durch den Ordner mit der Serverdatei <code>enshrouded_server.exe</code>:</p><code class="command">.\shroudforge\shroudforge.exe --root "D:\Games\EnshroudedServer\server" --desktop --target server</code><p>Aktiviere den Mod im ShroudForge-Fenster. Der Befehl öffnet die Servereinstellungen für den ausgewählten Ordner.</p></article><article><h3>Server starten</h3><p>Starte den Dedicated Server wie gewohnt über Steam oder dein Hosting-Panel. Der Server lädt Dateien aus seinem eigenen Installationsordner.</p></article><article><h3>Server-Log prüfen</h3><p>Servermeldungen stehen in <code>shroudforge/logs/shroudforge-server.log</code> im Serverordner.</p></article></section>
    <div class="callout info"><b>Keine Proton-Einstellung nötig</b><span><code>WINEDLLOVERRIDES</code> gilt nur für das Proton-Docker-Image. Für das Windows-Spiel und den Windows Dedicated Server aus Steam brauchst du sie nicht.</span></div>