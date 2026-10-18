# Live-Erkennung des im Menü ausgewählten Charakters

## Verfahren

Die Namensquelle ist der laufende Enshrouded-Client. Der Modloader sucht den Prozess dynamisch über `enshrouded.exe`. Beim ersten Aufruf wird der lesbare private Prozessspeicher im Hintergrund nach dem beobachteten Menüeintrag durchsucht. Die Suche verwendet keinen festen RVA und blockiert nicht den UI-Thread.

Ein Eintrag wird nur erkannt, wenn er die beobachtete Textstruktur erfüllt:

~~~text
[Steam]<Name> Stufe <Zahl> … Build-Version:<Zahl>
~~~

`Level` wird ebenfalls als Levelbeschriftung akzeptiert. Bei mehreren Treffern wird der Eintrag mit der niedrigsten dynamisch gefundenen Speicheradresse verwendet. Der A/B-Vergleich unten belegt, dass genau dieser Eintrag beim Wechsel der Menüauswahl seinen Namen wechselte, während eine weitere Textkopie unverändert blieb.

Nach dem ersten Treffer merkt sich der Prozess den dynamisch gefundenen Textanfang. Bei späteren UI-Aktualisierungen liest das Modul nur diesen kleinen Bereich erneut. Dadurch wird ein Wechsel auf einen anderen Charakter am selben Eintrag erkannt. Ein neuer Prozess, also ein Neustart des Spiels, verwirft die gespeicherte Adresse und startet die Suche erneut.

Wenn der Prozess fehlt, mehrere Clients laufen, der Speicher nicht lesbar ist oder der Eintrag nicht zur beobachteten Struktur passt, liefert das Modul keinen erkannten Namen. Die Oberfläche kann dann ihren konfigurierten Fallback verwenden. Ein alter Save-Indexname wird nicht als aktuelle Auswahl ausgegeben.

Wird in einem vollständigen Scan kein passender Eintrag gefunden, versucht das Modul es nach 60 Sekunden erneut, solange derselbe Spielprozess läuft.

## Live-Beleg

Am 18. Oktober 2026 wurde im laufenden Client PID `17872` der markierte Charakter im Menü von Ikiju auf Bonsaibauer gewechselt. Vor dem Wechsel enthielt der beobachtete Eintrag `Ikiju`, danach an derselben dynamisch gefundenen Position `Bonsaibauer`. Die angezeigten Stufen waren 41 und 37. Eine zweite Ikiju-Textkopie blieb nach dem Wechsel vorhanden.

Der Capture [`selection-menu-20261018-222020/live.json`](../../../dev/profiles/captures/selection-menu-20261018-222020/live.json) hält Prozess-ID, Client-Build und ausführbare Datei fest. Er enthält keinen konsistenten UI-Snapshot und belegt selbst nicht den Textwechsel. Der Textvergleich stammt aus der gezielten lesenden Live-Speicherbeobachtung vor und nach der vom Nutzer vorgenommenen Auswahländerung.

Der Save-Index blieb bei `latest: 4` und wurde durch die Menüauswahl nicht aktualisiert. Deshalb entfernt das Namensmodul die Auswahl anhand von `characters-index.latest` und `last_play_time`. Save-Dateien liefern keine aktive Menüauswahl.

## Grenzen

Die dynamische Speicheradresse ist nicht an eine Spielversion gebunden. Das Textmuster hängt jedoch von der UI-Darstellung und den Markierungen `[Steam]` sowie `Build-Version:` ab. Eine Änderung dieser Darstellung führt zu keinem Treffer und damit zum Fallback.

Die Auswahl des ersten Treffers ist durch einen A/B-Wechsel im aktuellen Client belegt. Sie ist noch nicht über weitere Client-Builds und alle möglichen Speicherlayouts hinweg bestätigt. Der Leser bleibt deshalb read-only und gibt bei fehlender passender Struktur keinen Save-basierten Ersatznamen aus.
