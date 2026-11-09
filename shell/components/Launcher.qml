import QtQuick
import QtQuick.Controls.Basic
import Quickshell
import ".."
import "../theme.js" as Theme

// The launcher (Super+Shift+D): the Start menu's search alone, near the top
// of the focused screen. The same search (Search) and the same rows: type,
// then Return starts the first program, opens the task or the file, or runs
// what was typed. Up and Down pick a row; Escape or a click outside closes
// it.
//
// Under the field, a list: before anything is typed, the programs started
// last (Search.recent), then every program; while typing, the results. The
// sides and the bottom edge resize the panel, and the size is kept
// (Settings.launcherWidth, launcherHeight: the list's height). A list of
// height 0 is the field alone: then the results show under it while typing,
// only as tall as they are.
Item {
    id: launcher

    signal dismissed

    /// The size of the screen it shows on, to keep the panel inside it.
    property real screenWidth: 1920
    property real screenHeight: 1080

    /// Called by the window each time it opens.
    function reset() {
        field.text = "";
        launcher.pick = 0;
        Search.query = "";
        field.forceActiveFocus();
        results.positionViewAtBeginning();
    }

    /// The row Return chooses.
    property int pick: 0

    readonly property int headerHeight: Theme.menuSearchHeight + 8 + Theme.menuPadding * 2
    /// The list's height: 0 for the field alone.
    readonly property int listHeight: Math.max(0, Math.min(Settings.launcherHeight, launcher.maxListHeight))
    readonly property bool compact: launcher.listHeight === 0
    /// The panel ends above the taskbar: it never runs off the screen.
    readonly property int maxListHeight: Math.max(0, Math.round(launcher.screenHeight * 0.82)
        - Theme.taskbarHeight - launcher.headerHeight - Theme.menuPadding - 8)
    /// The most rows the compact panel grows to while typing.
    readonly property int compactRows: 14

    readonly property int panelWidth: Math.max(360, Math.min(Settings.launcherWidth,
        Math.round(launcher.screenWidth * 0.9)))
    /// The window's height: the tallest the panel gets, so the window is not
    /// resized at each key.
    readonly property int panelHeight: launcher.headerHeight + Theme.menuPadding
        + (launcher.compact ? launcher.compactRows * StartMenuState.rowHeight : launcher.listHeight)

    function accept() {
        if (Search.accept(launcher.pick))
            launcher.dismissed();
    }

    function movePick(by) {
        launcher.pick = Math.max(0, Math.min(Search.pickCount - 1, launcher.pick + by));
    }

    onPickChanged: {
        var index = Search.indexOfPick(launcher.pick);
        if (index >= 0)
            results.positionViewAtIndex(index, ListView.Contain);
    }

    // A click outside the panel closes it.
    MouseArea {
        anchors.fill: parent
        acceptedButtons: Qt.LeftButton | Qt.RightButton | Qt.MiddleButton
        onPressed: launcher.dismissed()
    }

    Rectangle {
        id: panel

        /// The compact panel shows its rows only while typing, as tall as
        /// they are.
        readonly property int compactRowsShown: Search.active
            ? Math.min(Search.rows.length + (Search.pickCount === 0 ? 1 : 0), launcher.compactRows) : 0

        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: parent.top
        width: launcher.panelWidth
        height: launcher.headerHeight + (launcher.compact
            ? (panel.compactRowsShown > 0 ? panel.compactRowsShown * StartMenuState.rowHeight + Theme.menuPadding : 0)
            : launcher.listHeight + Theme.menuPadding)
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
                    results.positionViewAtBeginning();
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
                Keys.onUpPressed: launcher.movePick(-1)
                Keys.onDownPressed: launcher.movePick(1)
                Keys.onPressed: function (event) {
                    var page = Math.max(1, Math.floor(results.height / StartMenuState.rowHeight) - 1);
                    if (event.key === Qt.Key_A && (event.modifiers & Qt.ControlModifier)) {
                        field.selectAll();
                    } else if (event.key === Qt.Key_PageDown) {
                        launcher.movePick(page);
                    } else if (event.key === Qt.Key_PageUp) {
                        launcher.movePick(-page);
                    } else {
                        return;
                    }
                    event.accepted = true;
                }
            }
        }

        // The rows: a ListView makes only the ones in sight, so the list of
        // every program costs no more than a screenful.
        ListView {
            id: results

            anchors.left: parent.left
            anchors.right: parent.right
            anchors.top: searchBox.bottom
            anchors.bottom: parent.bottom
            anchors.margins: Theme.menuPadding
            visible: launcher.compact ? Search.active : true
            clip: true
            boundsBehavior: Flickable.StopAtBounds
            model: Search.rows
            ScrollBar.vertical: AeroScrollBar { id: resultsBar }

            readonly property real rowWidth: width - (resultsBar.visible ? resultsBar.width + 2 : 0)

            delegate: MenuRow {
                required property var modelData
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
                selected: row.pick >= 0 && row.pick === launcher.pick
                onActivated: {
                    Search.choose(row);
                    launcher.dismissed();
                }
            }

            // With nothing found, Return runs the text, as Run... does.
            footer: MenuRow {
                width: results.rowWidth
                visible: Search.active && Search.pickCount === 0
                height: visible ? StartMenuState.rowHeight : 0
                label: "Run “" + Search.query.trim() + "”"
                icon: Qt.resolvedUrl("../assets/icons/run.png")
                selected: true
                onActivated: launcher.accept()
            }
        }

        // ----- the edges that resize it -----------------------------------
        //
        // The panel is centred, with its top fixed: a side sets the width to
        // twice the pointer's distance from the middle, the bottom edge sets
        // the list's height from the pointer's height. The edge stays under
        // the pointer.

        component Edge: MouseArea {
            /// Where the pointer is in the launcher's own coordinates.
            function at(mouse) {
                return mapToItem(launcher, mouse.x, mouse.y);
            }
            hoverEnabled: true
        }

        Edge {
            anchors.left: parent.left
            anchors.top: parent.top
            anchors.bottom: parent.bottom
            width: 8
            cursorShape: Qt.SizeHorCursor
            onPositionChanged: function (mouse) {
                if (pressed)
                    Settings.launcherWidth = Math.round(2 * (launcher.width / 2 - at(mouse).x));
            }
        }

        Edge {
            anchors.right: parent.right
            anchors.top: parent.top
            anchors.bottom: parent.bottom
            width: 8
            cursorShape: Qt.SizeHorCursor
            onPositionChanged: function (mouse) {
                if (pressed)
                    Settings.launcherWidth = Math.round(2 * (at(mouse).x - launcher.width / 2));
            }
        }

        Edge {
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            anchors.leftMargin: 8
            anchors.rightMargin: 8
            height: 8
            cursorShape: Qt.SizeVerCursor
            onPositionChanged: function (mouse) {
                if (pressed)
                    Settings.launcherHeight = Math.max(0, Math.min(launcher.maxListHeight,
                        Math.round(at(mouse).y - launcher.headerHeight - Theme.menuPadding)));
            }
        }
    }
}
