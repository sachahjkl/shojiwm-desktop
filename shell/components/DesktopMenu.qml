pragma ComponentBehavior: Bound

import QtQuick
import ".."
import "../theme.js" as Theme

// A context menu of the desktop, drawn like a Windows 7 menu, from a plain
// list of lines:
//
//   { label, icon, enabled, checked, children: [...], run: function () {} }
//   { separator: true }
//
// `checked` puts a mark in front of the label (true or false makes the line
// checkable). A line with `children` opens this same component beside it,
// as deep as the list goes.
//
// It is an item, not a popup: ShojiWM hit-tests the windows before the
// popups of a background surface, so a popup of the desktop would be drawn
// over a window and let the window take its clicks. The shell draws it in a
// full-screen surface above everything instead (shell.qml).
Item {
    id: popup

    property var entries: []
    /// Where the menu hangs from: the pointer, or the right edge of the line
    /// of the parent menu. `flipX` is where it goes when there is no room.
    property real atX: 0
    property real atY: 0
    property real flipX: atX
    /// The area the menu must stay inside.
    property real boundsWidth: 0
    property real boundsHeight: 0

    /// A line was chosen: close the whole menu.
    signal dismissed

    /// The line whose submenu is open, or null.
    property var openEntry: null
    property real openX: 0
    property real openY: 0
    property real openFlipX: 0

    onEntriesChanged: popup.openEntry = null

    visible: entries.length > 0

    width: Math.min(
        Theme.popupMaxWidth,
        Math.max(Theme.popupMinWidth, popup.widest + Theme.popupPadX * 2 + 16 + 7 + 24))
    height: column.implicitHeight + Theme.popupPadY * 2 + 2
    // To the right of where it hangs from and down, or over to the left and
    // up when the screen ends.
    x: popup.atX + width <= boundsWidth ? popup.atX : Math.max(0, popup.flipX - width)
    y: popup.atY + height <= boundsHeight ? popup.atY : Math.max(0, boundsHeight - height)

    FontMetrics {
        id: metrics
        font.family: Theme.fontFamily
        font.pixelSize: Theme.fontSize
    }

    readonly property real widest: {
        var width = 0;
        for (var i = 0; i < entries.length; i++)
            width = Math.max(width, metrics.advanceWidth(entries[i].label || ""));
        return width;
    }

    Rectangle {
        anchors.fill: parent
        radius: Scheme.popupRadius
        border.width: 1
        border.color: Scheme.popupOutline
        gradient: Gradient {
            GradientStop { position: 0.0; color: Scheme.popupTop }
            GradientStop { position: 1.0; color: Scheme.popupBottom }
        }
    }

    // The light line inside the outline, as every Windows 7 menu has.
    Rectangle {
        anchors.fill: parent
        anchors.margins: 1
        radius: Math.max(0, Scheme.popupRadius - 1)
        color: "transparent"
        border.width: 1
        border.color: Scheme.popupInnerLine
    }

    property var restingEntry: null
    property var restingLine: null

    Timer {
        id: submenuTimer
        interval: Theme.submenuDelay
        onTriggered: popup.openFrom(popup.restingEntry, popup.restingLine)
    }

    function openFrom(entry, line) {
        submenuTimer.stop();
        if (!entry || !line) {
            popup.openEntry = null;
            return;
        }
        var at = line.mapToItem(popup, 0, 0);
        popup.openX = popup.x + popup.width - 2;
        popup.openFlipX = popup.x + 2;
        popup.openY = popup.y + at.y - Theme.popupPadY - 1;
        popup.openEntry = entry;
    }

    // The open submenu: this same component, beside its line. It lives in
    // the parent of this menu, so both share one set of coordinates.
    Loader {
        id: child

        active: popup.openEntry !== null
        parent: popup.parent
        z: popup.z + 1
        // By file: QML refuses a component that names itself.
        source: "DesktopMenu.qml"
        onLoaded: {
            var menu = child.item;
            menu.entries = Qt.binding(function () {
                return popup.openEntry ? popup.openEntry.children : [];
            });
            menu.atX = Qt.binding(function () { return popup.openX; });
            menu.atY = Qt.binding(function () { return popup.openY; });
            menu.flipX = Qt.binding(function () { return popup.openFlipX; });
            menu.boundsWidth = Qt.binding(function () { return popup.boundsWidth; });
            menu.boundsHeight = Qt.binding(function () { return popup.boundsHeight; });
        }
    }

    Connections {
        target: child.item
        ignoreUnknownSignals: true
        function onDismissed() { popup.dismissed(); }
    }

    Column {
        id: column

        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.topMargin: Theme.popupPadY
        anchors.leftMargin: 1
        anchors.rightMargin: 1

        Repeater {
            model: popup.entries

            delegate: MenuItemRow {
                required property var modelData

                id: line

                width: column.width
                separator: modelData.separator === true
                itemEnabled: modelData.enabled !== false
                label: modelData.label || ""
                icon: modelData.icon || ""
                hasChildren: modelData.children !== undefined
                highlighted: popup.openEntry === modelData
                checkable: modelData.checked !== undefined
                checked: modelData.checked === true

                onHoveredChanged: {
                    if (!line.hovered)
                        return;
                    if (line.hasChildren && line.itemEnabled) {
                        popup.restingEntry = modelData;
                        popup.restingLine = line;
                        if (popup.openEntry !== modelData)
                            submenuTimer.restart();
                    } else {
                        submenuTimer.stop();
                        popup.openEntry = null;
                    }
                }

                onActivated: {
                    if (line.hasChildren) {
                        popup.openFrom(modelData, line);
                        return;
                    }
                    // Read before closing: closing empties the list and
                    // destroys this line.
                    var run = modelData.run;
                    popup.dismissed();
                    if (run)
                        run();
                }
            }
        }
    }
}
