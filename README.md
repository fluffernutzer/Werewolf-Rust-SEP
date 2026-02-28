# Werwolf in Rust 
Mit der Anwendung soll das Spiel Werwolf online, mithilfe von Websockets spielbar sein. 

## Wie startet man die Anwendung? 
1) cargo run
2) Link aus der Kommandozeile im Browser öffnen
3) Zwischen 3 und 16 Spieler hinzufügen (Für Testzwecke sind 6-8 empfehlenswert, da sonst zwischen vielen Tabs gewechselt werden muss)
4) Klicke auf "Alle Spieler Bereit setzen". Daraufhin öffnet sich für jeden Spieler ein Tab
5) Für alle Spieler muss der "Verstanden" Button gedrückt werden, damit das tatsächliche Spiel startet
6) Welche jeweilige Rolle aktuell am Zug ist lässt sich an der Phase ablesen. Für das Lynchen am Tag und die Werwolfaktion müssen jeweils alle lebenden Spieler bzw. alle lebenden Werwölfe für ein Opfer abstimmen.
7) Das Spiel läuft bis es ein Siegerteam gibt.
   
## Implementierte Features
- Basis-Spiel "Werwolf"
- Möglichkeit auf dem Handy zu spielen (Anmeldung via QR-Code)
- verschiedene Rollen mit verschiedenen Aktionen:
    - Dorfbewohner,
    - Werwolf,
    - Seher,
    - Hexe,
    - Jäger,
    - Amor,
    - Doktor,
    - Priester
- Abstimmungsfunktion
- Chatfunktion
  
## Verwendung von KI  
|KI  | Nutzung |
| ----------- | ----------- |
| ChatGPT | Erstellung einer Farbpalette für das Frontend, Generieren von erzählerischen Texten |
| Deepseek | Recherchezwecke, Hilfestellung für die Korrektur von Bugs und Konflikten |
|LeChat(MistralAI)|Darstellungsverbesserung in user.html, Recherchezwecke|
|Microsoft Copilot|Recherchezwecke|
