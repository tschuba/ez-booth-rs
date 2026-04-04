# Data Backup Guide / Leitfaden Datensicherung

This guide explains how EZ Booth backup and recovery works for event operators.

Dieser Leitfaden erklaert, wie Datensicherung und Wiederherstellung in EZ Booth fuer Veranstaltungs-Teams funktionieren.

## 1. Where Your Data Lives / Wo Ihre Daten gespeichert werden

### EN

- EZ Booth stores event data in the current browser on the current device.
- The app does not automatically sync data to a server or cloud storage.
- If browser storage is cleared, local event data can be removed.
- Backups are JSON files that you download and store outside the browser.

### DE

- EZ Booth speichert Veranstaltungsdaten im aktuellen Browser auf dem aktuellen Geraet.
- Die App synchronisiert Daten nicht automatisch mit einem Server oder Cloud-Speicher.
- Wenn Browserdaten geloescht werden, koennen lokale Veranstaltungsdaten verloren gehen.
- Backups sind JSON-Dateien, die Sie herunterladen und ausserhalb des Browsers speichern.

## 2. What Can Cause Data Loss / Was zu Datenverlust fuehren kann

### EN

Data can be lost if you:

- clear browser history, website data, or storage
- switch to a different browser profile
- move to another laptop or tablet without exporting first
- use private browsing and close the session
- reset or replace the device

### DE

Daten koennen verloren gehen, wenn Sie:

- Browserverlauf, Webseitendaten oder Speicher loeschen
- zu einem anderen Browser-Profil wechseln
- auf ein anderes Notebook oder Tablet wechseln, ohne vorher zu exportieren
- einen privaten Browsermodus verwenden und die Sitzung schliessen
- das Geraet zuruecksetzen oder austauschen

## 3. Full Backup: Export Everything / Vollstaendiges Backup: Alles exportieren

### EN

Use a full backup when you want to protect all booths, vendors, and purchases.

Steps:

1. Open the booth list page.
2. Click `Export All`.
3. Save the downloaded JSON file to a safe location.
4. Confirm the file exists outside the browser, for example in `Downloads`, a team folder, or external storage.

Typical filename:

- `ez-booth-backup-2026-03-29.json`

### DE

Verwenden Sie ein vollstaendiges Backup, wenn Sie alle Veranstaltungen, Verkaeufer und Kaeufe sichern moechten.

Schritte:

1. Oeffnen Sie die Standliste.
2. Klicken Sie auf `Alle exportieren`.
3. Speichern Sie die heruntergeladene JSON-Datei an einem sicheren Ort.
4. Pruefen Sie, dass die Datei ausserhalb des Browsers vorhanden ist, zum Beispiel in `Downloads`, einem Team-Ordner oder auf einem externen Speichermedium.

Typischer Dateiname:

- `ez-booth-backup-2026-03-29.json`

## 4. Booth Backup: Export One Event / Stand-Backup: Eine Veranstaltung exportieren

### EN

Use a booth backup when you only need one event.

Steps:

1. Open the booth list.
2. Find the booth card you want to protect.
3. Click the booth export action on that card.
4. Save the downloaded JSON file.

Typical filename:

- `ez-booth-spring-market-2026-2026-03-29.json`

### DE

Verwenden Sie ein Stand-Backup, wenn Sie nur eine einzelne Veranstaltung sichern moechten.

Schritte:

1. Oeffnen Sie die Standliste.
2. Suchen Sie die Standkarte der gewuenschten Veranstaltung.
3. Klicken Sie auf die Export-Aktion dieses Standes.
4. Speichern Sie die heruntergeladene JSON-Datei.

Typischer Dateiname:

- `ez-booth-spring-market-2026-2026-03-29.json`

## 5. Import A Backup / Ein Backup importieren

### EN

You can import either a full backup or a booth backup.

Steps:

1. Open the booth list page.
2. Click `Import`.
3. Select a `.json` backup file.
4. Review the preview shown by EZ Booth.
5. Choose how conflicts should be handled.
6. Apply the import.
7. Verify the booth list, vendors, and purchases afterwards.

Conflict strategies:

- `Merge`: import new records, prefer strictly newer booth and purchase records, and keep the richer vendor name when multiple devices exported the same booth
- `Skip`: keep existing records and ignore conflicting imported ones
- `Replace`: overwrite existing conflicting records with imported data

Important merge details:

- If the same booth or purchase already exists and the timestamps are exactly equal, EZ Booth keeps the existing local record.
- If the same vendor exists in both imports, EZ Booth keeps a non-empty vendor name and prefers the richer name if one device has more complete vendor text.
- If two devices created different purchases, both purchases are kept as long as they have different purchase IDs.

### DE

Sie koennen sowohl ein vollstaendiges Backup als auch ein Stand-Backup importieren.

Schritte:

1. Oeffnen Sie die Standliste.
2. Klicken Sie auf `Importieren`.
3. Waehlen Sie eine `.json`-Backup-Datei aus.
4. Pruefen Sie die von EZ Booth angezeigte Vorschau.
5. Waehlen Sie, wie Konflikte behandelt werden sollen.
6. Starten Sie den Import.
7. Pruefen Sie danach Standliste, Verkaeufer und Kaeufe.

Konfliktstrategien:

- `Merge`: importiert neue Datensaetze, bevorzugt bei Staenden und Kaeufen nur eindeutig neuere Datensaetze und behaelt bei Verkaeufern den aussagekraeftigeren Namen
- `Skip`: behaelt vorhandene Datensaetze und ignoriert konfligierende importierte Datensaetze
- `Replace`: ueberschreibt vorhandene konfligierende Datensaetze mit den importierten Daten

Wichtige Merge-Details:

- Wenn derselbe Stand oder Kauf bereits vorhanden ist und die Zeitstempel exakt gleich sind, behaelt EZ Booth den bereits lokalen Datensatz.
- Wenn derselbe Verkaeufer in beiden Importen vorkommt, behaelt EZ Booth einen nicht-leeren Verkaeufernamen und bevorzugt den aussagekraeftigeren Namen.
- Wenn zwei Geraete unterschiedliche Kaeufe erzeugt haben, bleiben beide Kaeufe erhalten, solange sie unterschiedliche Kauf-IDs haben.

## 5a. Multi-Device Booth Workflow / Mehrgeraete-Workflow fuer einzelne Staende

### EN

This is the main recommended workflow when one booth is worked on across multiple devices.

Recommended sequence:

1. Start from one known-good booth backup.
2. Import that booth backup on each additional device before entering more data.
3. Let each device record its own new purchases.
4. Export the booth again from each device.
5. Import those booth backups on the target device with `Merge`.
6. Verify the final booth totals, vendor list, and recent purchases.

Best practices:

- prefer booth backups for device-to-device booth work instead of full backups
- import before creating new data on another device whenever possible
- keep the exported files from each device until the merged result is verified
- after a successful merge, create one fresh booth backup as the new recovery point

Limits to understand:

- EZ Booth does not try to guess whether two different purchase IDs are actually the same real-world sale
- if two devices change the same booth or purchase at the same timestamp, the existing local record is kept during `Merge`
- multi-file imports are applied one after another, so verify the result after importing several files

### DE

Dies ist der empfohlene Hauptablauf, wenn ein einzelner Stand auf mehreren Geraeten bearbeitet wird.

Empfohlene Reihenfolge:

1. Starten Sie mit einem bekannten gueltigen Stand-Backup.
2. Importieren Sie dieses Stand-Backup auf jedem weiteren Geraet, bevor neue Daten erfasst werden.
3. Lassen Sie jedes Geraet seine neuen Kaeufe erfassen.
4. Exportieren Sie den Stand anschliessend erneut von jedem Geraet.
5. Importieren Sie diese Stand-Backups auf dem Zielgeraet mit `Merge`.
6. Pruefen Sie die finalen Stand-Summen, die Verkaeuferliste und die letzten Kaeufe.

Empfohlene Praxis:

- verwenden Sie fuer Geraete-zu-Geraete-Standarbeit bevorzugt Stand-Backups statt Voll-Backups
- importieren Sie nach Moeglichkeit immer zuerst, bevor auf einem weiteren Geraet neue Daten erfasst werden
- bewahren Sie die Exportdateien aller Geraete auf, bis das Merge-Ergebnis geprueft ist
- erstellen Sie nach einem erfolgreichen Merge ein neues Stand-Backup als neuen Wiederherstellungspunkt

Wichtige Grenzen:

- EZ Booth versucht nicht zu erraten, ob zwei unterschiedliche Kauf-IDs denselben realen Verkauf meinen
- wenn zwei Geraete denselben Stand oder Kauf mit exakt gleichem Zeitstempel aendern, bleibt beim `Merge` der bereits lokale Datensatz erhalten
- mehrere Dateien werden nacheinander importiert; pruefen Sie deshalb das Ergebnis nach dem Import mehrerer Dateien

## 6. When To Create Backups / Wann Backups erstellt werden sollten

### EN

Create a backup:

- before clearing browser data
- before browser updates or profile cleanup
- before moving to another device
- before and after an event day
- before testing imports or larger cleanup actions
- after important data entry sessions

### DE

Erstellen Sie ein Backup:

- vor dem Loeschen von Browserdaten
- vor Browser-Updates oder Profilbereinigungen
- vor dem Wechsel auf ein anderes Geraet
- vor und nach einem Veranstaltungstag
- vor Test-Importen oder groesseren Bereinigungen
- nach wichtigen Eingabephasen

## 7. Where To Store Backup Files / Wo Backup-Dateien gespeichert werden sollten

### EN

Recommended storage locations:

- a shared team folder
- an external USB drive
- an organization-managed cloud folder
- a second device controlled by the event team

Good practice:

- keep more than one copy
- include the event date in your folder structure
- avoid storing the only copy inside the browser environment

### DE

Empfohlene Speicherorte:

- ein gemeinsamer Team-Ordner
- ein externer USB-Datentraeger
- ein von der Organisation verwalteter Cloud-Ordner
- ein zweites Geraet des Veranstaltungsteams

Gute Praxis:

- bewahren Sie mehr als eine Kopie auf
- verwenden Sie die Veranstaltungsdaten in Ihrer Ordnerstruktur
- speichern Sie nicht die einzige Kopie nur innerhalb der Browser-Umgebung

## 8. After Data Loss / Nach einem Datenverlust

### EN

If data is missing:

1. Stop entering new event data until you know what was lost.
2. Check whether the issue is only a different browser profile or device.
3. Locate the newest relevant backup file.
4. Import the backup from the booth list.
5. Verify booth counts, vendor lists, and recent purchases.
6. Create a fresh backup after successful recovery.

### DE

Wenn Daten fehlen:

1. Erfassen Sie keine neuen Veranstaltungsdaten, bis klar ist, was verloren ging.
2. Pruefen Sie, ob nur ein anderes Browser-Profil oder Geraet verwendet wird.
3. Suchen Sie die neueste passende Backup-Datei.
4. Importieren Sie das Backup ueber die Standliste.
5. Pruefen Sie Standanzahl, Verkaeuferlisten und aktuelle Kaeufe.
6. Erstellen Sie nach erfolgreicher Wiederherstellung ein neues Backup.

## 9. Quick Recommendation / Kurze Empfehlung

### EN

Before each event session, export a full backup and confirm the file was saved outside the browser.

### DE

Exportieren Sie vor jeder Veranstaltungssitzung ein vollstaendiges Backup und pruefen Sie, dass die Datei ausserhalb des Browsers gespeichert wurde.
