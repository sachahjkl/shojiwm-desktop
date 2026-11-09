import QtQuick
import Quickshell
import ".."
import "../theme.js" as Theme

// The launcher (Super+Shift+D): the Start menu's search alone, in the middle
// of the main screen. The same search (Search) and the same rows: type, then
// Return starts the first program, opens the task or the file, or runs what
// was typed. Up and Down pick a row; Escape or a click outside closes it.
Item {
    id: launcher

    signal dismissed

    /// Called by the window each time it opens.
    function reset() {
        field.text = "";
        launcher.pick = 0;
        Search.query = "";
        field.forceActiveFocus();
    }

    /// The row Return chooses.
    property int pick: 0
    /// The most rows the panel shows.
    readonly property int maxRows: 14
    /// The tallest the panel gets: the window is this tall, so it is not
    /// resized at each key.
    readonly property int fullHeight: Theme.menuSearchHeight + 8 + Theme.menuPadding * 3
        + launcher.maxRows * StartMenuState.rowHeight

    function accept() {
        if (Search.accept(launcher.pick))
            launcher.dismissed();
    }

    // A click outside the panel closes it.
    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.LeftButton | Qt.RightButton | Qt.MiddleButton
        onPressed: launcher.dismissed()
    }

    Rectangle {
        id: panel

        readonly property int rowsShown: Math.min(Search.rows.length
            + (Search.active && Search.pickCount === 0 ? 1 : 0), launcher.maxRows)

        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: parent.top
        width: parent.width
        height: searchBox.height + Theme.menuPadding * 2
            + (Search.active ? panel.rowsShown * StartMenuState.rowHeight + Theme.menuPadding : 0)
        radius: Scheme.popupRadius + 1
        color: Scheme.startGlassOutline

        // The click on the panel itself stays here.
        MouseArea {
            anchors.fill: parent
        }

        Rectangle {
            anchors.fill: parent
            anchors.margins: 1
            radius: Scheme.popupRadius
            gradient: Gradient {
                GradientStop { position: 0.0; color: Scheme.menuTop }
                GradientStop { position: 1.0; color: Scheme.menuBottom }
            }
        }

        Rectangle {
            id: searchBox

            anchors.left: parent.left
            anchors.right: parent.right
            anchors.top: parent.top
            anchors.margins: Theme.menuPadding
            height: Theme.menuSearchHeight + 8
            radius: Scheme.fieldRadius
            color: Scheme.field
            border.width: 1
            border.color: field.activeFocus ? Scheme.fieldFocus : Scheme.fieldBorder

            Image {
                id: searchIcon
                anchors.left: parent.left
                anchors.leftMargin: 8
                anchors.verticalCenter: parent.verticalCenter
                width: 16
                height: 16
                source: "../assets/search.svg"
                sourceSize.width: 32
                sourceSize.height: 32
                smooth: true
            }

            TextInput {
                id: field

                anchors.left: searchIcon.right
                anchors.leftMargin: 8
                anchors.right: parent.right
                anchors.rightMargin: 8
                anchors.verticalCenter: parent.verticalCenter
                color: Scheme.text
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSize + 2
                selectByMouse: true
                focus: true
                clip: true

                onTextChanged: {
                    launcher.pick = 0;
                    Search.query = text;
                }

                Text {
                    anchors.fill: parent
                    verticalAlignment: Text.AlignVCenter
                    text: "Search programs, settings and files, or type a command"
                    color: Scheme.placeholder
                    font: field.font
                    elide: Text.ElideRight
                    visible: field.text.length === 0
                }

                Keys.onEscapePressed: {
                    if (field.text.length > 0)
                        field.text = "";
                    else
                        launcher.dismissed();
                }
                Keys.onReturnPressed: launcher.accept()
                Keys.onEnterPressed: launcher.accept()
                Keys.onUpPressed: {
                    if (launcher.pick > 0)
                        launcher.pick -= 1;
                }
                Keys.onDownPressed: {
                    if (launcher.pick < Search.pickCount - 1)
                        launcher.pick += 1;
                }
                Keys.onPressed: function (event) {
                    if (event.key === Qt.Key_A && (event.modifiers & Qt.ControlModifier)) {
                        field.selectAll();
                        event.accepted = true;
                    }
                }
            }
        }

        ListView {
            id: results

            anchors.left: parent.left
            anchors.right: parent.right
            anchors.top: searchBox.bottom
            anchors.bottom: parent.bottom
            anchors.margins: Theme.menuPadding
            visible: Search.active
            clip: true
            interactive: false
            model: Search.rows

            delegate: MenuRow {
                required property var modelData
                readonly property var row: modelData
                width: results.width
                heading: row.kind === "heading"
                label: row.kind === "heading" ? row.label
                    : row.kind === "program" ? row.value.name
                    : row.kind === "task" ? row.value.text
                    : row.value.substring(row.value.lastIndexOf("/") + 1)
                icon: row.kind === "program" ? Apps.entryIcon(row.value)
                    : row.kind === "task" ? Qt.resolvedUrl("../assets/icons/control-panel.png")
                    : row.kind === "file" ? Apps.fileIcon(row.value) : ""
                selected: row.pick >= 0 && row.pick === launcher.pick
                onActivated: {
                    Search.choose(row);
                    launcher.dismissed();
                }
            }

            // With nothing found, Return runs the text, as Run... does.
            footer: MenuRow {
                width: results.width
                visible: Search.pickCount === 0
                height: visible ? StartMenuState.rowHeight : 0
                label: "Run “" + Search.query.trim() + "”"
                icon: Qt.resolvedUrl("../assets/icons/run.png")
                selected: true
                onActivated: launcher.accept()
            }
        }
    }
}
