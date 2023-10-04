<h2 align="center">Bachelorarbeit zum Thema:</h2>
<h1 align="center">Effiziente Routenplanung in Straßennetzen mit Contraction Hierarchies</h1>

<p align="center"><b>Daniel Holzner</b></p>
<p align="center"><b>17.09.2023</b></p>

---

### Zusammenfassung
Diese Arbeit beschäftigt sich mit einem grundlegenden Problem der Graphentheorie: der
Berechnung kürzester Wege zwischen zwei Knoten in einem Graphen. Ein praktisches
Anwendungsgebiet hierfür ist die Routenplanung in Straßennetzen, bei der das Ziel darin
besteht, die zeitlich kürzeste Route zwischen zwei Orten zu ermitteln. Obwohl klassische
Algorithmen wie Dijkstra oder A⋆ in der Lage sind, diese Aufgabe zu bewältigen, stoßen
sie bei großen Graphen mit Millionen von Knoten und Kanten an ihre Grenzen, was ihre
Einsatzmöglichkeiten beispielsweise in Echtzeit-Navigation und standortbasierten Diensten
einschränkt.

Mit der Methode der Contraction Hierarchies, erstmals von Geisberger et al. vorgestellt,
können diese Einschränkungen überwunden werden. Hierbei wird der Graph zunächst
in einer einmaligen Vorberechnungsphase durch Hinzufügen von Abkürzungskanten mit
zusätzlichen Informationen erweitert. Diese Abkürzungskanten werden dann während der
Suchphase ausgenutzt, um die Suche zu beschleunigen. Das Ziel dieser Arbeit ist es, anhand
einer konkreten Implementierung dieser Technik zu zeigen, wie die Berechnung kürzester
Wege in Straßennetzen beschleunigt werden kann. Dabei sollen die Ergebnisse auch mit
konventionellen Techniken wie Dijkstra oder A⋆verglichen werden.

Die Implementierung wurde in der Programmiersprache Rust umgesetzt und ist als Bibliothek
verfügbar. Die Laufzeitanalyse zeigt eine deutliche Verbesserung der Ausführungsgeschwindigkeit
im Vergleich zu Dijkstra und A⋆. Alle Messungen wurden mit realen Straßendaten
des OpenData-Projekts OpenStreetMap durchgeführt. In großen Straßennetzen, vergleichbar
mit der Größe Deutschlands (10 Mio. Knoten und 22 Mio. Kanten), können Wegberechnungen
über große Entfernungen durch Nutzung von CHs in weniger als einer Millisekunde
durchgeführt werden, was einen Verbesserungsfaktor von mehr als 1000 gegenüber den
Standardverfahren ergibt.
