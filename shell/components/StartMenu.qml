pragma ComponentBehavior: Bound

import QtQuick
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
        search.text = "";
        menu.page = "home";
        menu.openFolder = "";
        menu.powerMenuOpen = false;
        menu.hoveredPlace = "";
        results.currentIndex = 0;
        search.forceActiveFocus();
    }

    onVisibleChanged: {
        if (!visible)
            menu.powerMenuOpen = false;
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

    // ----- the panel --------------------------------------------------------

    Rectangle {
        anchors.fill: parent
        radius: 5
        border.width: 1
        border.color: Theme.menuOutline
        gradient: Gradient {
            GradientStop { position: 0.0; color: Theme.menuTop }
            GradientStop { position: 1.0; color: Theme.menuBottom }
        }
    }

    // The places column, dark, down the right hand side.
    Rectangle {
        id: places

        anchors.right: parent.right
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        anchors.margins: 1
        width: menu.width - Theme.menuProgramsWidth - 2
        radius: 5
        gradient: Gradient {
            GradientStop { position: 0.0; color: Theme.menuPlacesTop }
            GradientStop { position: 1.0; color: Theme.menuPlacesBottom }
        }

        Column {
            id: placesColumn

            anchors.left: parent.left
            anchors.right: parent.right
            anchors.top: parent.top
            anchors.margins: Theme.menuPadding
            spacing: 2

            Item {
                width: parent.width
                height: 64

                // The user's picture, or the place the pointer is on.
                Item {
                    id: avatar
                    anchors.left: parent.left
                    anchors.verticalCenter: parent.verticalCenter
                    width: 44
                    height: 44

                    Image {
                        anchors.fill: parent
                        source: "../assets/user.svg"
                        sourceSize.width: 88
                        sourceSize.height: 88
                        smooth: true
                        opacity: menu.hoveredPlace.length > 0 ? 0 : 1
                        Behavior on opacity {
                            NumberAnimation { duration: Theme.fade }
                        }
                    }

                    // A framed tile, as the user's picture is.
                    Rectangle {
                        anchors.fill: parent
                        radius: 4
                        color: "#33ffffff"
                        border.width: 1
                        border.color: "#99ffffff"
                        opacity: menu.hoveredPlace.length > 0 ? 1 : 0
                        Behavior on opacity {
                            NumberAnimation { duration: Theme.fade }
                        }

                        Image {
                            anchors.centerIn: parent
                            width: 34
                            height: 34
                            source: menu.hoveredPlace.length > 0
                                ? "../assets/icons/" + menu.hoveredPlace + ".svg" : ""
                            sourceSize.width: 68
                            sourceSize.height: 68
                            smooth: true
                        }
                    }
                }

                // Cut short with an ellipsis, and kept off the right edge.
                Text {
                    anchors.left: avatar.right
                    anchors.leftMargin: 10
                    anchors.right: parent.right
                    anchors.rightMargin: Theme.menuPadding
                    anchors.verticalCenter: parent.verticalCenter
                    text: Apps.userName
                    color: Theme.textOnPlaces
                    font.family: Theme.fontFamily
                    font.pixelSize: Theme.fontSize + 1
                    elide: Text.ElideRight
                    maximumLineCount: 1
                }
            }

            Rectangle {
                width: parent.width
                height: 1
                color: Theme.menuDivider
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
                    { label: "Control Panel", icon: "place-settings.svg", tile: "control-panel",
                      command: Apps.controlPanel }
                ]

                delegate: Item {
                    id: place

                    required property var modelData

                    width: placesColumn.width
                    height: modelData.separator ? 9 : Theme.menuRowHeight

                    Rectangle {
                        anchors.verticalCenter: parent.verticalCenter
                        width: parent.width
                        height: 1
                        color: Theme.menuDivider
                        visible: place.modelData.separator === true
                    }

                    MenuRow {
                        anchors.fill: parent
                        visible: place.modelData.separator !== true
                        label: place.modelData.label || ""
                        icon: place.modelData.icon
                            ? Qt.resolvedUrl("../assets/" + place.modelData.icon) : ""
                        labelColor: Theme.textOnPlaces
                        onHoveredChanged: {
                            if (hovered)
                                menu.hoveredPlace = place.modelData.tile || "";
                            else if (menu.hoveredPlace === place.modelData.tile)
                                menu.hoveredPlace = "";
                        }
                        onActivated: {
                            if (place.modelData.command)
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
                hot: buttonArea.containsMouse
                pressed: buttonArea.pressed

                Row {
                    anchors.centerIn: parent
                    spacing: 6

                    Image {
                        anchors.verticalCenter: parent.verticalCenter
                        width: 16
                        height: 16
                        source: "../assets/power.svg"
                        sourceSize.width: 32
                        sourceSize.height: 32
                        smooth: true
                    }

                    Text {
                        anchors.verticalCenter: parent.verticalCenter
                        text: power.action ? power.action.label : ""
                        color: Theme.textOnMenu
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
                    color: Theme.textOnMenu
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
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        anchors.margins: 1
        width: Theme.menuProgramsWidth

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

            Column {
                id: homeColumn

                width: home.width
                spacing: 1

                Repeater {
                    model: Apps.pinned

                    delegate: MenuRow {
                        required property var modelData
                        width: homeColumn.width
                        label: modelData ? modelData.name : ""
                        icon: Apps.entryIcon(modelData)
                        bold: true
                        onActivated: menu.launch(modelData)
                    }
                }

                Item {
                    width: homeColumn.width
                    height: 9
                    visible: Apps.pinned.length > 0 && Apps.frequent.length > 0

                    Rectangle {
                        anchors.verticalCenter: parent.verticalCenter
                        width: parent.width
                        height: 1
                        color: Theme.menuDivider
                    }
                }

                Repeater {
                    model: Apps.frequent

                    delegate: MenuRow {
                        required property var modelData
                        width: homeColumn.width
                        label: modelData ? modelData.name : ""
                        icon: Apps.entryIcon(modelData)
                        onActivated: menu.launch(modelData)
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

            delegate: Column {
                id: folder

                required property var modelData
                readonly property bool expanded: menu.openFolder === modelData.name

                width: folderList.width

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
                        required property var modelData
                        width: folder.width
                        indent: 22
                        label: modelData ? modelData.name : ""
                        icon: Apps.entryIcon(modelData)
                        onActivated: menu.launch(modelData)
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

            header: MenuRow {
                width: results.width
                heading: true
                label: menu.found.length > 0 ? "Programs" : "No program matches"
            }

            delegate: MenuRow {
                required property var modelData
                required property int index
                width: results.width
                label: modelData ? modelData.name : ""
                icon: Apps.entryIcon(modelData)
                selected: index === results.currentIndex
                onActivated: menu.launch(modelData)
            }

            // With nothing found, Return runs the text, as Run... does.
            footer: MenuRow {
                width: results.width
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
                color: Theme.menuDivider
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
            radius: 3
            color: "#ffffff"
            border.width: 1
            border.color: search.activeFocus ? "#3f82b4" : "#8fa9bf"

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
                color: Theme.textOnMenu
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
                    color: "#7d95a9"
                    font: search.font
                    elide: Text.ElideRight
                    visible: search.text.length === 0
                }

                Keys.onEscapePressed: {
                    if (search.text.length > 0)
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
}
