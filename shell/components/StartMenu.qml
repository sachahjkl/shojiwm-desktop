pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls.Basic
import Quickshell
import Quickshell.Io
import ".."
import "../theme.js" as Theme

// The Start menu, laid out as Windows 7 lays it out.
//
// Left: the pinned programs, the ones used most under them, and "All
// Programs", which swaps the list for the programs in folders. Typing filters
// every program instead; the arrow keys move through the results, Return
// starts the one picked, and with nothing found Return runs what was typed.
//
// Right: the user, the places, and the power button, whose arrow opens log
// off, lock, restart, sleep and hibernate.
Item {
    id: menu

    signal dismissed

    /// "home" (pinned and frequent) or "all" (All Programs).
    property string page: "home"
    /// The folder open in All Programs, by name, or "".
    property string openFolder: ""
    property bool powerMenuOpen: false
    /// The colour icon of the place under the pointer, shown in the user's
    /// tile as Windows 7 does, or "" for the user's picture.
    property string hoveredPlace: ""

    /// Cleared by the window every time the menu is opened.
    function reset() {
        menu.programMenu = [];
        search.text = "";
        menu.page = "home";
        menu.openFolder = "";
        menu.powerMenuOpen = false;
        menu.hoveredPlace = "";
        menu.pick = 0;
        menu.filesFound = [];
        menu.jumpEntry = null;
        search.forceActiveFocus();
    }

    onVisibleChanged: {
        if (!visible) {
            menu.powerMenuOpen = false;
            menu.programMenu = [];
        }
    }

    // ----- the menu of a program, on a right click ------------------------------

    /// Its lines, or [] while it is closed, and where it hangs from.
    property var programMenu: []
    property point programMenuAt: Qt.point(0, 0)

    function showProgramMenu(row, entry, x, y) {
        if (!entry)
            return;
        var at = row.mapToItem(menu, x, y);
        menu.programMenuAt = Qt.point(at.x, at.y);
        var pinned = Apps.isPinned(entry);
        var onTaskbar = Apps.isTaskbarPinned(entry);
        menu.programMenu = [
            { label: "Open", run: function () { menu.launch(entry); } },
            { separator: true },
            { label: onTaskbar ? "Unpin from Taskbar" : "Pin to Taskbar",
              run: function () { Apps.setTaskbarPinned(entry, !onTaskbar); } },
            { label: pinned ? "Unpin from Start Menu" : "Pin to Start Menu",
              run: function () { Apps.setPinned(entry, !pinned); } }
        ];
    }

    // ----- the jump list of a program, in the right column ----------------------
    //
    // As Windows 7 does: resting on a program that has recent files shows
    // them in place of the places column; a click opens one with the program.

    property var jumpEntry: null
    readonly property var jumpPinned: menu.jumpEntry ? Apps.pinnedFilesFor(menu.jumpEntry) : []
    readonly property var jumpRecent: menu.jumpEntry ? Apps.recentUnpinned(menu.jumpEntry) : []

    Timer {
        id: jumpDelay
        property var entry: null
        interval: 350
        onTriggered: menu.jumpEntry = jumpDelay.entry
    }

    function hoverProgram(entry, inside) {
        if (!inside) {
            jumpDelay.stop();
            return;
        }
        if (Apps.recentFor(entry).length > 0 || Apps.pinnedFilesFor(entry).length > 0) {
            jumpDelay.entry = entry;
            jumpDelay.restart();
        } else {
            jumpDelay.stop();
            menu.jumpEntry = null;
        }
    }

    readonly property var found: Apps.search(search.text)
    readonly property bool searching: search.text.trim().length > 0
    /// The Control Panel tasks and the files that match, as Windows 7 lists
    /// them under the programs.
    readonly property var tasksFound: ControlPanelState.findTasks(search.text).slice(0, 5)
    property var filesFound: []

    /// The results as rows: a heading over each group that has any, then
    /// its rows. `pick` numbers the rows that can be chosen; -1 is a heading.
    readonly property var rows: {
        var out = [];
        var pick = 0;
        var group = function (title, kind, list) {
            if (list.length === 0)
                return;
            out.push({ kind: "heading", label: title, pick: -1 });
            for (var i = 0; i < list.length; i++)
                out.push({ kind: kind, value: list[i], pick: pick++ });
        };
        group("Programs", "program", menu.found);
        group("Control Panel", "task", menu.tasksFound);
        group("Files", "file", menu.filesFound);
        return out;
    }
    readonly property int pickCount: {
        var count = 0;
        for (var i = 0; i < menu.rows.length; i++) {
            if (menu.rows[i].pick >= 0)
                count++;
        }
        return count;
    }
    /// The row Return chooses.
    property int pick: 0

    // The files: fd in your home folder, a moment after the last key, for
    // names that hold what is typed. Hidden folders stay out.
    Timer {
        id: findFiles
        interval: 250
        onTriggered: {
            var query = search.text.trim();
            if (query.length < 2) {
                menu.filesFound = [];
                return;
            }
            fileFinder.running = false;
            fileFinder.command = ["fd", "--max-results", "8", "--ignore-case",
                                  "--fixed-strings", "--", query, Apps.home];
            fileFinder.running = true;
        }
    }

    Process {
        id: fileFinder
        stdout: StdioCollector {
            onStreamFinished: {
                menu.filesFound = String(text).split("\n").filter(function (line) {
                    return line.length > 0;
                }).map(function (line) { return line.replace(/\/$/, ""); });
            }
        }
    }

    function launch(entry) {
        Apps.launch(entry);
        menu.dismissed();
    }

    /// Choose a result row: start the program, open the task's page, or
    /// open the file.
    function choose(row) {
        if (!row)
            return;
        if (row.kind === "program") {
            menu.launch(row.value);
        } else if (row.kind === "task") {
            ControlPanelState.open(row.value.page);
            menu.dismissed();
        } else if (row.kind === "file") {
            // The file's own program; a folder opens in the file manager.
            Quickshell.execDetached(["xdg-open", row.value]);
            menu.dismissed();
        }
    }

    /// Return in the search box: the row picked, else run the text.
    function accept() {
        if (!menu.searching)
            return;
        for (var i = 0; i < menu.rows.length; i++) {
            if (menu.rows[i].pick === Math.min(menu.pick, menu.pickCount - 1)) {
                menu.choose(menu.rows[i]);
                return;
            }
        }
        if (Apps.runTyped(search.text))
            menu.dismissed();
    }

    // ----- the glass ----------------------------------------------------------
    //
    // As Windows 7 builds it: the whole menu is a pane of glass with the
    // frame's lines, the programs sit in a white pane set into it on the
    // left, and the places are white text on the glass on the right. The
    // account picture stands above the top edge.

    Item {
        id: glass

        anchors.fill: parent
        anchors.topMargin: Theme.menuTileOverhang

        Rectangle {
            anchors.fill: parent
            radius: Scheme.menuRadius + 1
            color: Scheme.startGlassOutline
        }

        Rectangle {
            anchors.fill: parent
            anchors.margins: 1
            radius: Scheme.menuRadius
            gradient: Gradient {
                GradientStop { position: 0.0; color: Scheme.startGlassTop }
                GradientStop { position: 0.55; color: Scheme.startGlassMiddle }
                GradientStop { position: 1.0; color: Scheme.startGlassBottom }
            }
            border.width: 1
            border.color: Scheme.startGlassHighlight
        }

        // The colour of the glass chosen in the Control Panel, over it.
        Rectangle {
            anchors.fill: parent
            anchors.margins: 1
            radius: Scheme.menuRadius
            visible: Settings.glassColor.length > 0
            color: Settings.glassColor.length > 0 ? Settings.glassColor : "transparent"
            opacity: Settings.glassColorStrength
        }
    }

    // The places column, dark, down the right hand side.
    Rectangle {
        id: places

        anchors.right: parent.right
        anchors.top: glass.top
        anchors.bottom: parent.bottom
        anchors.margins: 1
        width: menu.width - Theme.menuProgramsWidth - 2
        color: "transparent"

        // The jump list of the program under the pointer, in place of the
        // places.
        Column {
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.top: parent.top
            anchors.margins: Theme.menuPadding
            anchors.topMargin: Theme.menuPadding + Theme.menuTileSize - Theme.menuTileOverhang
            spacing: 1
            visible: menu.jumpEntry !== null && !menu.searching

            MenuRow {
                width: parent.width
                heading: true
                label: "Pinned"
                visible: menu.jumpPinned.length > 0
            }

            Repeater {
                model: menu.jumpPinned

                delegate: MenuRow {
                    required property var modelData
                    width: parent.width
                    label: modelData.name
                    icon: Apps.fileIcon(modelData.name)
                    labelColor: Scheme.placesText
                    pinState: "unpin"
                    onPinClicked: Apps.setFilePinned(menu.jumpEntry, modelData.path, false)
                    onActivated: {
                        Apps.openWith(menu.jumpEntry, modelData.path);
                        menu.dismissed();
                    }
                }
            }

            MenuRow {
                width: parent.width
                heading: true
                label: "Recent"
                visible: menu.jumpRecent.length > 0
            }

            Repeater {
                model: menu.jumpRecent

                delegate: MenuRow {
                    required property var modelData
                    width: parent.width
                    label: modelData.name
                    icon: Apps.fileIcon(modelData.name)
                    labelColor: Scheme.placesText
                    pinState: "pin"
                    onPinClicked: Apps.setFilePinned(menu.jumpEntry, modelData.path, true)
                    onActivated: {
                        Apps.openWith(menu.jumpEntry, modelData.path);
                        menu.dismissed();
                    }
                }
            }
        }

        Column {
            id: placesColumn

            anchors.left: parent.left
            anchors.right: parent.right
            anchors.top: parent.top
            anchors.margins: Theme.menuPadding
            spacing: 2
            visible: menu.jumpEntry === null || menu.searching

            // Room under the picture, which hangs over this corner.
            Item {
                width: parent.width
                height: Theme.menuTileSize - Theme.menuTileOverhang
            }

            // The account's name, the first line, as Windows 7 has it.
            PlaceRow {
                width: parent.width
                label: Apps.userName
                onActivated: {
                    Apps.openPath(Apps.home);
                    menu.dismissed();
                }
            }

            Repeater {
                model: [
                    { label: "Documents", icon: "place-documents.svg", tile: "documents",
                      path: Apps.userDir("DOCUMENTS", "Documents") },
                    { label: "Downloads", icon: "place-downloads.svg", tile: "downloads",
                      path: Apps.userDir("DOWNLOAD", "Downloads") },
                    { label: "Pictures", icon: "place-pictures.svg", tile: "pictures",
                      path: Apps.userDir("PICTURES", "Pictures") },
                    { label: "Music", icon: "place-music.svg", tile: "music",
                      path: Apps.userDir("MUSIC", "Music") },
                    { label: "Videos", icon: "place-videos.svg", tile: "videos",
                      path: Apps.userDir("VIDEOS", "Videos") },
                    { separator: true },
                    { label: "Computer", icon: "place-computer.svg", tile: "computer",
                      path: "/" },
                    { separator: true },
                    { label: "Control Panel", icon: "place-settings.svg", tile: "control-panel",
                      panel: true },
                    { label: "Run...", tile: "run", run: true }
                ]

                delegate: Item {
                    id: place

                    required property var modelData

                    width: placesColumn.width
                    height: modelData.separator ? 11 : Theme.menuRowHeight

                    // An etched line: dark over light, short of both edges.
                    Column {
                        anchors.centerIn: parent
                        visible: place.modelData.separator === true

                        Rectangle { width: place.width * 0.72; height: 1; color: "#4d000000" }
                        Rectangle { width: place.width * 0.72; height: 1; color: "#33ffffff" }
                    }

                    PlaceRow {
                        anchors.fill: parent
                        visible: place.modelData.separator !== true
                        label: place.modelData.label || ""
                        onHoveredChanged: {
                            if (hovered)
                                menu.hoveredPlace = place.modelData.tile || "";
                            else if (menu.hoveredPlace === place.modelData.tile)
                                menu.hoveredPlace = "";
                        }
                        onActivated: {
                            if (place.modelData.run) {
                                // Run... is the search box, which runs what
                                // it does not find.
                                search.forceActiveFocus();
                                return;
                            }
                            if (place.modelData.panel)
                                ControlPanelState.open("");
                            else if (place.modelData.command)
                                Apps.run(place.modelData.command);
                            else
                                Apps.openPath(place.modelData.path);
                            menu.dismissed();
                        }
                    }
                }
            }
        }

        // ----- the power button, in the corner --------------------------------

        Row {
            id: power

            anchors.right: parent.right
            anchors.bottom: parent.bottom
            anchors.margins: Theme.menuPadding
            height: 28

            readonly property var action: Apps.powerAction(Apps.powerButtonAction)

            PowerPart {
                width: 104
                leftRounded: true
                danger: true
                hot: buttonArea.containsMouse
                pressed: buttonArea.pressed

                // The label alone, as Windows 7 has it; Vista had the icon.
                Text {
                    anchors.centerIn: parent
                    text: power.action ? power.action.label : ""
                    color: "#ffffff"
                    font.family: Theme.fontFamily
                    font.pixelSize: Theme.fontSize
                }

                MouseArea {
                    id: buttonArea
                    anchors.fill: parent
                    hoverEnabled: true
                    onClicked: {
                        menu.dismissed();
                        Apps.runPower(Apps.powerButtonAction);
                    }
                }
            }

            PowerPart {
                id: arrow

                width: 20
                rightRounded: true
                danger: true
                hot: arrowArea.containsMouse || menu.powerMenuOpen
                pressed: arrowArea.pressed || menu.powerMenuOpen

                // A small solid triangle, 4 by 7 px, as Windows 7 draws it.
                Canvas {
                    anchors.centerIn: parent
                    width: 4
                    height: 7
                    onPaint: {
                        var ctx = getContext("2d");
                        ctx.reset();
                        ctx.fillStyle = "#f2ffffff";
                        ctx.beginPath();
                        ctx.moveTo(0, 0);
                        ctx.lineTo(width, height / 2);
                        ctx.lineTo(0, height);
                        ctx.closePath();
                        ctx.fill();
                    }
                }

                MouseArea {
                    id: arrowArea
                    anchors.fill: parent
                    hoverEnabled: true
                    onClicked: menu.powerMenuOpen = !menu.powerMenuOpen
                }
            }
        }
    }

    PowerMenu {
        id: powerMenu

        anchorItem: arrow
        open: menu.powerMenuOpen && menu.visible
        onDismissed: menu.powerMenuOpen = false
    }

    // A menu with no outside presses closes once the pointer has left both
    // it and its arrow.
    Timer {
        interval: 900
        running: menu.powerMenuOpen
        repeat: true
        onTriggered: {
            if (!powerMenu.pointerInside && !arrowArea.containsMouse)
                menu.powerMenuOpen = false;
        }
    }

    // ----- the programs column ----------------------------------------------

    Item {
        id: programs

        anchors.left: parent.left
        anchors.top: glass.top
        anchors.bottom: parent.bottom
        anchors.margins: Theme.menuGlassBorder
        width: Theme.menuProgramsWidth - Theme.menuGlassBorder

        // The white pane, set into the glass with the frame's two lines.
        Rectangle {
            anchors.fill: parent
            anchors.margins: -2
            radius: Scheme.popupRadius + 2
            color: "#99ffffff"

            Rectangle {
                anchors.fill: parent
                anchors.margins: 1
                radius: Scheme.popupRadius + 1
                color: "#a6000000"
            }
        }
        Rectangle {
            anchors.fill: parent
            radius: Scheme.popupRadius
            gradient: Gradient {
                GradientStop { position: 0.0; color: Scheme.menuTop }
                GradientStop { position: 1.0; color: Scheme.menuBottom }
            }
        }

        // --- pinned, then used most -------------------------------------------

        Flickable {
            id: home

            anchors.left: parent.left
            anchors.right: parent.right
            anchors.top: parent.top
            anchors.bottom: allPrograms.top
            anchors.margins: Theme.menuPadding
            visible: !menu.searching && menu.page === "home"
            clip: true
            contentHeight: homeColumn.implicitHeight
            boundsBehavior: Flickable.StopAtBounds
            ScrollBar.vertical: AeroScrollBar { id: homeBar }

            Column {
                id: homeColumn

                width: home.width - (homeBar.visible ? homeBar.width + 2 : 0)
                spacing: 1

                Repeater {
                    model: Apps.pinned

                    delegate: MenuRow {
                        id: pinnedRow
                        required property var modelData
                        width: homeColumn.width
                        label: modelData ? modelData.name : ""
                        icon: Apps.entryIcon(modelData)
                        bold: true
                        onActivated: menu.launch(modelData)
                        onMenuRequested: (x, y) => menu.showProgramMenu(pinnedRow, modelData, x, y)
                        dragId: Apps.entryFileId(modelData)
                        hasArrow: Apps.recentFor(modelData).length > 0
                            || Apps.pinnedFilesFor(modelData).length > 0
                        selected: menu.jumpEntry === modelData
                        onHoveredChanged: menu.hoverProgram(modelData, hovered)
                    }
                }

                Item {
                    width: homeColumn.width
                    height: 9
                    visible: Apps.pinned.length > 0 && Apps.frequent.length > 0
                        && Settings.showFrequent

                    Rectangle {
                        anchors.verticalCenter: parent.verticalCenter
                        width: parent.width
                        height: 1
                        color: Scheme.menuDivider
                    }
                }

                Repeater {
                    model: Settings.showFrequent ? Apps.frequent : []

                    delegate: MenuRow {
                        id: frequentRow
                        required property var modelData
                        width: homeColumn.width
                        label: modelData ? modelData.name : ""
                        icon: Apps.entryIcon(modelData)
                        onActivated: menu.launch(modelData)
                        onMenuRequested: (x, y) => menu.showProgramMenu(frequentRow, modelData, x, y)
                        dragId: Apps.entryFileId(modelData)
                        hasArrow: Apps.recentFor(modelData).length > 0
                            || Apps.pinnedFilesFor(modelData).length > 0
                        selected: menu.jumpEntry === modelData
                        onHoveredChanged: menu.hoverProgram(modelData, hovered)
                    }
                }
            }
        }

        // --- All Programs, in folders -----------------------------------------

        ListView {
            id: folderList

            anchors.left: parent.left
            anchors.right: parent.right
            anchors.top: parent.top
            anchors.bottom: allPrograms.top
            anchors.margins: Theme.menuPadding
            visible: !menu.searching && menu.page === "all"
            clip: true
            spacing: 1
            boundsBehavior: Flickable.StopAtBounds
            model: Apps.folders
            ScrollBar.vertical: AeroScrollBar { id: foldersBar }

            delegate: Column {
                id: folder

                required property var modelData
                readonly property bool expanded: menu.openFolder === modelData.name

                width: folderList.width - (foldersBar.visible ? foldersBar.width + 2 : 0)

                MenuRow {
                    width: folder.width
                    label: folder.modelData.name
                    icon: Qt.resolvedUrl(folder.expanded
                        ? "../assets/icons/folder-open.png" : "../assets/icons/folder.png")
                    selected: folder.expanded
                    onActivated: menu.openFolder =
                        folder.expanded ? "" : folder.modelData.name
                }

                Repeater {
                    model: folder.expanded ? folder.modelData.entries : []

                    delegate: MenuRow {
                        id: folderRow
                        required property var modelData
                        width: folder.width
                        indent: 22
                        label: modelData ? modelData.name : ""
                        icon: Apps.entryIcon(modelData)
                        onActivated: menu.launch(modelData)
                        onMenuRequested: (x, y) => menu.showProgramMenu(folderRow, modelData, x, y)
                        dragId: Apps.entryFileId(modelData)
                    }
                }
            }
        }

        // --- search results ---------------------------------------------------

        ListView {
            id: results

            anchors.left: parent.left
            anchors.right: parent.right
            anchors.top: parent.top
            anchors.bottom: searchBox.top
            anchors.margins: Theme.menuPadding
            visible: menu.searching
            clip: true
            model: menu.rows
            boundsBehavior: Flickable.StopAtBounds
            ScrollBar.vertical: AeroScrollBar { id: resultsBar }
            readonly property real rowWidth: width - (resultsBar.visible ? resultsBar.width + 2 : 0)

            delegate: MenuRow {
                id: resultRow
                required property var modelData
                required property int index
                readonly property var row: modelData
                width: results.rowWidth
                heading: row.kind === "heading"
                label: row.kind === "heading" ? row.label
                    : row.kind === "program" ? row.value.name
                    : row.kind === "task" ? row.value.text
                    : row.value.substring(row.value.lastIndexOf("/") + 1)
                icon: row.kind === "program" ? Apps.entryIcon(row.value)
                    : row.kind === "task" ? Qt.resolvedUrl("../assets/icons/control-panel.png")
                    : row.kind === "file" ? Apps.fileIcon(row.value) : ""
                selected: row.pick >= 0 && row.pick === menu.pick
                onActivated: menu.choose(row)
                onMenuRequested: (x, y) => {
                    if (row.kind === "program")
                        menu.showProgramMenu(resultRow, row.value, x, y);
                }
                dragId: row.kind === "program" ? Apps.entryFileId(row.value) : ""
            }

            // With nothing found, Return runs the text, as Run... does.
            footer: MenuRow {
                width: results.rowWidth
                visible: menu.pickCount === 0
                height: visible ? Theme.menuRowHeight : 0
                label: "Run “" + search.text.trim() + "”"
                icon: Qt.resolvedUrl("../assets/icons/run.png")
                selected: true
                onActivated: menu.accept()
            }
        }

        // --- All Programs / Back ----------------------------------------------

        Item {
            id: allPrograms

            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: searchBox.top
            anchors.leftMargin: Theme.menuPadding
            anchors.rightMargin: Theme.menuPadding
            anchors.bottomMargin: 4
            height: menu.searching ? 0 : Theme.menuRowHeight + 9
            visible: !menu.searching

            Rectangle {
                anchors.top: parent.top
                anchors.topMargin: 4
                width: parent.width
                height: 1
                color: Scheme.menuDivider
            }

            MenuRow {
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.bottom: parent.bottom
                label: menu.page === "all" ? "◂  Back" : "All Programs  ▸"
                bold: true
                onActivated: {
                    menu.page = menu.page === "all" ? "home" : "all";
                    menu.openFolder = "";
                }
            }
        }

        // ----- search --------------------------------------------------------

        Rectangle {
            id: searchBox

            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            anchors.margins: Theme.menuPadding
            height: Theme.menuSearchHeight
            radius: Scheme.fieldRadius
            color: Scheme.field
            border.width: 1
            border.color: search.activeFocus ? Scheme.fieldFocus : Scheme.fieldBorder

            Image {
                id: searchIcon
                anchors.left: parent.left
                anchors.leftMargin: 6
                anchors.verticalCenter: parent.verticalCenter
                width: 14
                height: 14
                source: "../assets/search.svg"
                sourceSize.width: 28
                sourceSize.height: 28
                smooth: true
            }

            TextInput {
                id: search

                anchors.left: searchIcon.right
                anchors.leftMargin: 6
                anchors.right: parent.right
                anchors.rightMargin: 6
                anchors.verticalCenter: parent.verticalCenter
                color: Scheme.text
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSize
                selectByMouse: true
                focus: true
                clip: true

                onTextChanged: {
                    menu.pick = 0;
                    menu.jumpEntry = null;
                    findFiles.restart();
                }

                Text {
                    anchors.fill: parent
                    verticalAlignment: Text.AlignVCenter
                    text: "Search programs, or type a command"
                    color: Scheme.placeholder
                    font: search.font
                    elide: Text.ElideRight
                    visible: search.text.length === 0
                }

                Keys.onEscapePressed: {
                    if (menu.programMenu.length > 0)
                        menu.programMenu = [];
                    else if (search.text.length > 0)
                        search.text = "";
                    else
                        menu.dismissed();
                }
                Keys.onReturnPressed: menu.accept()
                Keys.onEnterPressed: menu.accept()
                // Ctrl+A picks the whole text, as in any Windows field.
                Keys.onPressed: function (event) {
                    if (event.key === Qt.Key_A && (event.modifiers & Qt.ControlModifier)) {
                        search.selectAll();
                        event.accepted = true;
                    }
                }
                Keys.onUpPressed: {
                    if (menu.searching && menu.pick > 0)
                        menu.pick -= 1;
                }
                Keys.onDownPressed: {
                    if (menu.searching && menu.pick < menu.pickCount - 1)
                        menu.pick += 1;
                }
            }
        }
    }

    // ----- the account picture, over the top edge -------------------------------

    Item {
        id: avatar

        x: places.x + (places.width - width) / 2
        y: 0
        width: Theme.menuTileSize
        height: Theme.menuTileSize

        UserTile {
            anchors.fill: parent
            source: Settings.userPictureSource
            dark: Settings.dark
            opacity: menu.hoveredPlace.length > 0 ? 0 : 1
            Behavior on opacity {
                NumberAnimation { duration: Settings.motion(Theme.fade) }
            }
        }

        // The place under the pointer, in the same frame, as Windows 7
        // shows it.
        UserTile {
            anchors.fill: parent
            source: menu.hoveredPlace.length > 0
                ? Qt.resolvedUrl("../assets/icons/" + menu.hoveredPlace + ".png") : ""
            dark: Settings.dark
            opacity: menu.hoveredPlace.length > 0 ? 1 : 0
            Behavior on opacity {
                NumberAnimation { duration: Settings.motion(Theme.fade) }
            }
        }
    }

    // A press outside the program menu closes it.
    MouseArea {
        anchors.fill: parent
        z: 10
        visible: menu.programMenu.length > 0
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        onPressed: menu.programMenu = []
    }

    DesktopMenu {
        z: 11
        entries: menu.programMenu
        atX: menu.programMenuAt.x
        atY: menu.programMenuAt.y
        boundsWidth: menu.width
        boundsHeight: menu.height
        onDismissed: menu.programMenu = []
    }
}
