pragma ComponentBehavior: Bound

import QtQuick
import ".."
import "../theme.js" as Theme

// The Start menu: programs on the left with a search box under them, places
// on the right, and the shut-down button in the corner.
Item {
    id: menu

    signal dismissed

    /// Cleared by the window every time the menu is opened.
    function reset() {
        search.text = "";
        search.forceActiveFocus();
    }

    readonly property var results: Apps.search(search.text)
    readonly property bool searching: search.text.length > 0

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

                Image {
                    id: avatar
                    anchors.left: parent.left
                    anchors.verticalCenter: parent.verticalCenter
                    width: 44
                    height: 44
                    source: "../assets/user.svg"
                    sourceSize.width: 88
                    sourceSize.height: 88
                    smooth: true
                }

                Text {
                    anchors.left: avatar.right
                    anchors.leftMargin: 8
                    anchors.verticalCenter: parent.verticalCenter
                    text: Apps.home.split("/").pop()
                    color: Theme.textOnPlaces
                    font.family: Theme.fontFamily
                    font.pixelSize: Theme.fontSize + 1
                }
            }

            Rectangle {
                width: parent.width
                height: 1
                color: Theme.menuDivider
            }

            MenuRow {
                width: parent.width
                label: "Documents"
                icon: "../assets/place-documents.svg"
                labelColor: Theme.textOnPlaces
                onActivated: {
                    Apps.openPath(Apps.home + "/Documents");
                    menu.dismissed();
                }
            }
            MenuRow {
                width: parent.width
                label: "Pictures"
                icon: "../assets/place-pictures.svg"
                labelColor: Theme.textOnPlaces
                onActivated: {
                    Apps.openPath(Apps.home + "/Pictures");
                    menu.dismissed();
                }
            }
            MenuRow {
                width: parent.width
                label: "Music"
                icon: "../assets/place-music.svg"
                labelColor: Theme.textOnPlaces
                onActivated: {
                    Apps.openPath(Apps.home + "/Music");
                    menu.dismissed();
                }
            }
            MenuRow {
                width: parent.width
                label: "Computer"
                icon: "../assets/place-computer.svg"
                labelColor: Theme.textOnPlaces
                onActivated: {
                    Apps.openPath("/");
                    menu.dismissed();
                }
            }
            MenuRow {
                width: parent.width
                label: "Control Panel"
                icon: "../assets/place-settings.svg"
                labelColor: Theme.textOnPlaces
                onActivated: {
                    Apps.run(Apps.controlPanel);
                    menu.dismissed();
                }
            }
        }

        // Shut down, in the corner.
        Rectangle {
            id: shutdown

            anchors.right: parent.right
            anchors.bottom: parent.bottom
            anchors.margins: Theme.menuPadding
            width: 128
            height: 28
            radius: 3
            border.width: 1
            border.color: Theme.menuOutline
            gradient: Gradient {
                GradientStop {
                    position: 0.0
                    color: shutdownArea.containsMouse ? "#ffffff" : "#eef6ff"
                }
                GradientStop { position: 1.0; color: "#c3d7e8" }
            }

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
                    text: "Shut down"
                    color: Theme.textOnMenu
                    font.family: Theme.fontFamily
                    font.pixelSize: Theme.fontSize
                }
            }

            MouseArea {
                id: shutdownArea
                anchors.fill: parent
                hoverEnabled: true
                onClicked: {
                    menu.dismissed();
                    Apps.run(["systemctl", "poweroff"]);
                }
            }
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

        Column {
            id: pinnedColumn

            anchors.left: parent.left
            anchors.right: parent.right
            anchors.top: parent.top
            anchors.margins: Theme.menuPadding
            spacing: 1
            visible: !menu.searching

            Repeater {
                model: Apps.pinned

                delegate: MenuRow {
                    required property var modelData
                    width: pinnedColumn.width
                    label: modelData ? modelData.name : ""
                    icon: Apps.entryIcon(modelData)
                    onActivated: {
                        Apps.launch(modelData);
                        menu.dismissed();
                    }
                }
            }

            Rectangle {
                width: parent.width
                height: 1
                color: Theme.menuDivider
                visible: Apps.pinned.length > 0
            }
        }

        ListView {
            id: list

            anchors.left: parent.left
            anchors.right: parent.right
            anchors.top: pinnedColumn.visible ? pinnedColumn.bottom : parent.top
            anchors.bottom: searchBox.top
            anchors.margins: Theme.menuPadding
            clip: true
            model: menu.results
            currentIndex: 0
            boundsBehavior: Flickable.StopAtBounds

            delegate: MenuRow {
                required property var modelData
                required property int index
                width: list.width
                label: modelData ? modelData.name : ""
                icon: Apps.entryIcon(modelData)
                selected: menu.searching && index === 0
                onActivated: {
                    Apps.launch(modelData);
                    menu.dismissed();
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

                Text {
                    anchors.fill: parent
                    verticalAlignment: Text.AlignVCenter
                    text: "Search programs"
                    color: "#7d95a9"
                    font: search.font
                    visible: search.text.length === 0
                }

                Keys.onEscapePressed: menu.dismissed()
                Keys.onReturnPressed: menu.launchFirst()
                Keys.onEnterPressed: menu.launchFirst()
            }
        }
    }

    /// Start the first program the search found.
    function launchFirst() {
        var found = menu.results;
        if (found.length > 0) {
            Apps.launch(found[0]);
            menu.dismissed();
        }
    }
}
