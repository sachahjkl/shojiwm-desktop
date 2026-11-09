pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls.Basic
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
        results.currentIndex = 0;
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

    readonly property var found: Apps.search(search.text)
    readonly property bool searching: search.text.trim().length > 0

    function launch(entry) {
        Apps.launch(entry);
        menu.dismissed();
    }

    /// Return in the search box: the program picked, else run the text.
    function accept() {
        if (!menu.searching)
            return;
        var found = menu.found;
        if (found.length > 0) {
            var index = Math.max(0, Math.min(results.currentIndex, found.length - 1));
            menu.launch(found[index]);
            return;
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

        Column {
            id: placesColumn

            anchors.left: parent.left
            anchors.right: parent.right
            anchors.top: parent.top
            anchors.margins: Theme.menuPadding
            spacing: 2

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
                glass: true
                hot: buttonArea.containsMouse
                pressed: buttonArea.pressed

                Row {
                    anchors.centerIn: parent
                    spacing: 6

                    Image {
                        anchors.verticalCenter: parent.verticalCenter
                        width: 16
                        height: 16
                        source: "../assets/power-white.svg"
                        sourceSize.width: 32
                        sourceSize.height: 32
                        smooth: true
                    }

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        text: power.action ? power.action.label : ""
                        color: "#ffffff"
                        font.family: Theme.fontFamily
                        font.pixelSize: Theme.fontSize
                    }
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

                width: 24
                rightRounded: true
                hot: arrowArea.containsMouse || menu.powerMenuOpen
                pressed: arrowArea.pressed || menu.powerMenuOpen

                Text {
                    anchors.centerIn: parent
                    text: "▸"
                    color: "#ffffff"
                    font.family: Theme.fontFamily
                    font.pixelSize: Theme.fontSize
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
                        ? "../assets/icons/folder-open.svg" : "../assets/icons/folder.svg")
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
            model: menu.found
            currentIndex: 0
            boundsBehavior: Flickable.StopAtBounds
            ScrollBar.vertical: AeroScrollBar { id: resultsBar }
            readonly property real rowWidth: width - (resultsBar.visible ? resultsBar.width + 2 : 0)

            header: MenuRow {
                width: results.rowWidth
                heading: true
                label: menu.found.length > 0 ? "Programs" : "No program matches"
            }

            delegate: MenuRow {
                id: resultRow
                required property var modelData
                required property int index
                width: results.rowWidth
                label: modelData ? modelData.name : ""
                icon: Apps.entryIcon(modelData)
                selected: index === results.currentIndex
                onActivated: menu.launch(modelData)
                onMenuRequested: (x, y) => menu.showProgramMenu(resultRow, modelData, x, y)
            }

            // With nothing found, Return runs the text, as Run... does.
            footer: MenuRow {
                width: results.rowWidth
                visible: menu.found.length === 0
                height: visible ? Theme.menuRowHeight : 0
                label: "Run “" + search.text.trim() + "”"
                icon: Qt.resolvedUrl("../assets/icons/run.svg")
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

                onTextChanged: results.currentIndex = 0

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
                Keys.onUpPressed: {
                    if (menu.searching && results.currentIndex > 0)
                        results.currentIndex -= 1;
                }
                Keys.onDownPressed: {
                    if (menu.searching && results.currentIndex < menu.found.length - 1)
                        results.currentIndex += 1;
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
                ? Qt.resolvedUrl("../assets/icons/" + menu.hoveredPlace + ".svg") : ""
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
