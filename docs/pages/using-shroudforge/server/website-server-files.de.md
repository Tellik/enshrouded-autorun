---
{
  "id": "website-server-files",
  "locale": "de",
  "title": "Serverdateien installieren",
  "summary": "Vorhandene Anleitung zum Herunterladen und Kopieren der Serverdateien.",
  "view": "server-files",
  "navigation": {
    "path": [
      "QUICKSTART"
    ],
    "label": "Serverdateien"
  }
}
---


    <div class="page-intro"><div class="eyebrow">QUICKSTART · MULTIPLAYER</div><h1>ShroudForge-Dateien<br>auf dem Server installieren.</h1><p class="lead">Diese gemeinsamen Schritte gelten für Windows- und Proton-Docker-Server. Danach wählst du die passende Serveranleitung im Quickstart-Menü.</p></div>
    <section class="server-docker-guide"><article><h3>Release herunterladen und entpacken</h3><p>Stoppe den Server und sichere die Spielwelt. Lade die <a href="https://github.com/bonsaibauer/shroudforge/releases/latest">aktuelle ShroudForge-Version</a> herunter. Klicke mit der rechten Maustaste auf die ZIP-Datei und wähle <b>Alle extrahieren</b>. Lass die entpackten Dateien zusammen.</p></article><article><h3>Serverinstallationsordner öffnen</h3><p>Suche den Ordner mit <code>enshrouded_server.exe</code>. ShroudForge kommt direkt in diesen Ordner neben die Serverdatei.</p><p>Beim Proton-Docker-Image findest du den Hostordner über die Anleitung <a href="#server-docker" data-view="server-docker">Proton-Docker</a>. Im Image <code>mornedhels/enshrouded-server</code> liegt die Serverdatei normalerweise im Unterordner <code>server</code> des eingebundenen Ordners. Prüfe den Pfad, bevor du Dateien kopierst.</p></article><article><h3>Alle ShroudForge-Dateien kopieren</h3><p>Kopiere <b>alle Dateien und Ordner</b> aus dem entpackten Release in den Ordner mit <code>enshrouded_server.exe</code>. Der Release enthält DLL-Dateien und den Ordner <code>shroudforge</code>. Erlaube das Kopieren in den Zielordner, wenn du danach gefragt wirst.</p></article><article><h3>Mod auf den Server kopieren</h3><p>Entpacke den Mod-Download und kopiere den Mod-Ordner in <code>mods</code> im selben Serverordner. Folge den Hinweisen des Mod-Erstellers zu weiteren Dateien oder zur Installation auf dem Spiel-PC.</p></article></section>
    <div class="path-cards"><button data-view="server-windows"><b>WINDOWS</b><strong>Windows Dedicated Server</strong><small>Server-Mod-Einstellungen öffnen und Server starten.</small><span>Anleitung öffnen →</span></button><button data-view="server-docker"><b>PROTON · DOCKER</b><strong>Container neu erstellen</strong><small>Vorhandene Einstellungen übernehmen und die ShroudForge-Variable ergänzen.</small><span>Anleitung öffnen →</span></button></div>