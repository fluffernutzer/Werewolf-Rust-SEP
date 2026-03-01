# Werwolf in Rust 
Implementation von Werwolf als asynchroner WebSocket-Server mit Axum und Tokio. Flexible Teilnahme über mobiles Gerät oder direkt am Server über Browser-Tabs.

## Wie startet man die Anwendung? 
1) cargo run
2) Link aus der Kommandozeile im Browser öffnen.
3) Beitreten über QR-Code oder Eingabe direkt am Server.
4) Zwischen 3 und 16 Spieler hinzufügen (Für Testzwecke sind 6-8 empfehlenswert, da sonst zwischen vielen Tabs gewechselt werden muss).
5) Auf "Alle Spieler Bereit setzen" klicken. Daraufhin öffnet sich für jeden Spieler ein Tab.
6) Für alle Spieler muss der "Verstanden" Button gedrückt werden, damit das tatsächliche Spiel startet.
7) Welche jeweilige Rolle aktuell am Zug ist lässt sich an der Phase ablesen. Für das Lynchen am Tag und die Werwolfaktion müssen jeweils alle lebenden Spieler bzw. alle lebenden Werwölfe für ein Opfer abstimmen.
8) Das Spiel läuft bis es ein Siegerteam gibt.
9) Spiel zurücksetzen, bringt alles auf ein leeren Ausgangspunkt zurück.
10) Spiel beenden, beendet den Server.
   
## Implementierte Features
### Beitritt
Nach Beitritt wird man in der Liste der teilnehemnden Spieler auf der Startseite geführt. Der Beitritt ist auf zwei verschiedene Arten möglich:
#### QR Code Beitritt
Scannen von QR Code führt Spieler zu einer seperaten join-Seite. Auf dieser kann er sich anmelden und verbeleibt in einem Warteraum, bis das Spiel beginnt. Jedem Spieler mit einem externen Gerät wird ein individueller Token zugewiesen, durch den eine korrekte Zuordnung jederzeit möglich ist.
#### Beitritt am Server
Eingeben von Namen direkt am Server. Bei Spielbeginn wird automatisch für jeden Spieler der so beigetreten ist ein eigener tab geöffnet.
### Spielbeginn
Zum einen müssen auf der Startseite alle Spieler auf bereit gesetzt werden und zum anderen muss dann jeder Spieler noch auf seiner persöhnlichen Seite angeben, dass er verstanden hat.
### Rollen und Aktionen
    - Dorfbewohner
    - Werwolf
    - Seher
    - Hexe
    - Jäger
    - Amor
    - Doktor
    - Priester
Je nach Rolle werden individuelle Aktionsbutton angezeigt.
### Phasenwechsel
Erfolgt automatisch anhand eines Enums, nach abschließen der aktuellen Phase ohne zutun der Spieler.
### Abstimmungsfunktion
Tagsüber können alle lebenden Spieler abstimmen, wen sie lynchen wollen. Nachts können die Werwölfe während ihrer Phase abstimmen, wen sie töten wollen.
### Chatfunktion.
Alle Spieler können sich über den Chat miteinander unterhalten, ungachtet dessen ob sie mit einem Tab oder ihrem eigenen Gerät teilnehmen.
### Anzeige
Alle Spieler bekommen jewils ihre Rolle, sowie eine kurze Beschreibung dieser angezeigt. Außerdem verfügt jeder Spieler Seite über eine Anzeige, aller Spieler sowie ob sie noch am leben oder bereits tot sind. Das Liebespaar erhält zudem noch ein Herz um sich dessen bewusst zu sein.
### Spiel zurücksetzen und beenden
Jederzeit möglich, ungeachtet der Spielphase.
#### Spiel zurücksetzen 
Löst ResetGame aus. 
Dabei werden alle Rollen, Phasen und Spielerlisten wieder auf null bzw leer gesetzt. Der Server bleibt aber weiterhin bestehen.
#### Spiel beenden
Löst EndGame aus.
Durch einen Oneshot-Channel wird ein GracefulShutdown ausgelöst. Der Server wird gestoppt, dabei werden alle Websocket verbindungen getrennt und das Programm beendet sich vollständig.

## Testing
Zum einen wurde das Programm mehrfach mit mehren Clients gleichzeitig auf die Funktionalität all seiner Features geprüft. Dabei wurden sowhol Tabs als auch mobile geräte genutzt um sicherzustellen, dass Abläufe synchron und Aktionen korrekt verarbeitet werden.
Zum anderen wurden auch mehrere Unit-test mit Cargo test durchgeführt. Dabei wurden zum einen einzelne Methoden und Abläufe, als auch ganze Spiele auf ihre Korrektheit geprüft. 
Durch die Kombination von manuellen und automatisierten Tests wurden sowohl das Zusammenspiel der Komponenten als auch die korrekte Funktionsweise einzelner Methoden überprüft.

## Verwendung von KI  
|KI  | Nutzung |
| ----------- | ----------- |
| ChatGPT | Erstellung einer Farbpalette für das Frontend, Generieren von erzählerischen Texten, Unterstützung bei Debugging |
| Deepseek | Recherchezwecke, Hilfestellung für die Korrektur von Bugs und Konflikten |
|LeChat(MistralAI)|Darstellungsverbesserung in user.html, Recherchezwecke|
|Microsoft Copilot|Recherchezwecke|
