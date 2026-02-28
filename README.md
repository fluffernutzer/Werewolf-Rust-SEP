Mit der Anwendung soll das Spiel Werwolf online, mithilfe von Websockets spielbar sein. 
Um die Anwendung zu starten: 
1) cargo run
2) Link aus der Kommandozeile im Browser öffnen
3) Zwischen 3 und 16 Spieler hinzufügen (Für Testzwecke sind 6-8 empfehlenswert, da sonst zwischen vielen Tabs gewechselt werden muss)
4) Klicke auf "Alle Spieler Bereit setzen". Daraufhin öffnet sich für jeden Spieler ein Tab
5) Für alle Spieler muss der "Verstanden" Button gedrückt werden, damit das tatsächliche Spiel startet
6) Welche jeweilige Rolle aktuell am Zug ist lässt sich an der Phase ablesen. Für das Lynchen am Tag und die Werwolfaktion müssen jeweils alle lebenden Spieler bzw. alle lebenden Werwölfe für ein Opfer abstimmen.
7) Das Spiel läuft bis es ein Siegerteam gibt.



Für die Implementierung wurde LeChat(MistralAI) verwendet(Hauptsächlich für Darstellungsverbesserungen in der Datei user.html, sowie Recherchezwecke).
