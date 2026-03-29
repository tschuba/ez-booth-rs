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

- `Merge`: prefer the newer matching record when EZ Booth can compare timestamps safely
- `Skip`: keep existing records and ignore conflicting imported ones
- `Replace`: overwrite existing conflicting records with imported data

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

- `Merge`: bevorzugt den neueren passenden Datensatz, wenn EZ Booth Zeitstempel sicher vergleichen kann
- `Skip`: behaelt vorhandene Datensaetze und ignoriert konfligierende importierte Datensaetze
- `Replace`: ueberschreibt vorhandene konfligierende Datensaetze mit den importierten Daten

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
